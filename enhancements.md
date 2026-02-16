# Matrix IPTV CLI — World-Class Performance Enhancement Specification

> **Goal**: Make Matrix IPTV CLI the fastest, leanest, and most responsive terminal IPTV client in the world while retaining 100% of existing functionality.
>
> **Audience**: AI agents and developers executing this spec. Every section is self-contained with file paths, data structures, and exact implementation details.
>
> **Execution Order**: Phases are numbered by priority. Each phase can be completed independently but should be merged in order. Do not skip phases.

---

## Implementation Status (Last Updated: 2026-02-15)

| Phase | Description                           | Status             | Notes                                                                                                                                                                                                                                                            |
| ----- | ------------------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | Local Catalog Cache                   | ✅ **COMPLETE**    | implemented in `src/cache.rs` and `src/handlers/async_actions.rs`.                                                                                                                                                                                               |
| 2     | List Virtualization Hardening         | ⚠️ **PARTIAL**     | Windowed rendering exists via `.skip().take()` but no standardized `visible_window()` helper. ListState offset management could be improved.                                                                                                                     |
| 3     | Incremental Search Narrowing          | ❌ **NOT STARTED** | `SearchState` lacks `narrowing_stack`. `update_search()` still performs full scans on every keystroke.                                                                                                                                                           |
| 4     | Typed API Models (FlexId)             | ✅ **COMPLETE**    | `FlexId` implemented in `src/flex_id.rs`. `api.rs` structs migrated. Downstream callsites updated.                                                                                                                                                               |
| 5     | Replace Mega-Regex with Lookup Tables | ✅ **COMPLETE**    | [`FOREIGN_KEYWORDS`](src/parser.rs:7) and [`FOREIGN_COUNTRY_CODES`](src/parser.rs:65) HashSets implemented with O(1) lookups. [`matches_foreign()`](src/parser.rs:338) replaces mega-regex. [`FOREIGN_VOD_KEYWORDS`](src/parser.rs:177) added for VOD filtering. |
| 6     | App Struct Decomposition              | ❌ **NOT STARTED** | [`App`](src/app.rs:107) struct is still monolithic (~95 fields). No `ContentState`, `SessionState`, `UiState` sub-structs.                                                                                                                                       |
| 7     | Index-Based Filtered Views            | ❌ **NOT STARTED** | Still using `Vec<Arc<Stream>>` for display lists. No index-based `Vec<usize>` views.                                                                                                                                                                             |
| 8     | Provider Profile System               | ❌ **NOT STARTED** | No `ProviderProfile` struct. Hardcoded provider logic still in preprocessing.rs.                                                                                                                                                                                 |
| 9     | Structured Error Handling             | ⚠️ **PARTIAL**     | `IptvError` enum exists but lacks `is_retryable()` method. No `with_retry()` wrapper. Still using `Option<String>` for error fields.                                                                                                                             |
| 10    | Comprehensive Test Suite              | ⚠️ **PARTIAL**     | Some inline tests exist in parser.rs and app.rs. Missing dedicated test files: `parser_tests.rs`, `preprocessing_tests.rs`, `cache_tests.rs`, `api_types_tests.rs`.                                                                                              |

### Current Architecture Observations

1. **App Struct Size**: The [`App`](src/app.rs:107) struct contains approximately 95 fields in a single flat structure, making it difficult to:
   - Understand which fields relate to which feature
   - Pass focused state to functions
   - Enable contributors to work on isolated subsystems

2. **Search Performance**: The [`update_search()`](src/app.rs:991) method performs full parallel scans on every keystroke:

   ```rust
   // Current: Full scan on every keystroke
   self.categories = self.all_categories.par_iter()
       .filter(|c| { /* ... */ })
       .cloned()
       .collect();
   ```

3. **Mega-Regex Still Present**: The [`FOREIGN_PATTERNS_REGEX`](src/parser.rs:63) combines 150+ patterns into a single regex, causing expensive backtracking.

4. **serde_json::Value Usage**: Stream IDs and other fields still use `serde_json::Value`:

   ```rust
   // Current in api.rs
   pub stream_id: serde_json::Value,
   pub num: Option<serde_json::Value>,
   pub rating: Option<serde_json::Value>,
   ```

5. **No Caching Layer**: Every app launch re-fetches the full catalog from the provider API. No local cache exists.

### Recommended Implementation Priority

Based on impact and effort:

1. **High Priority, High Impact**: Phase 1 (Cache) - Immediate user-visible improvement
2. **High Priority, Medium Effort**: Phase 5 (Regex → HashSet) - Quick performance win
3. **Medium Priority, High Effort**: Phase 6 (App Decomposition) - Enables future development
4. **Medium Priority**: Phase 4 (Typed Models) - Reduces memory and improves type safety
5. **Lower Priority**: Phases 2, 3, 7, 8, 9, 10 - Incremental improvements

---

## Table of Contents

1. [Phase 1: Local Catalog Cache — Instant Cold Starts](#phase-1-local-catalog-cache--instant-cold-starts)
2. [Phase 2: List Virtualization Hardening](#phase-2-list-virtualization-hardening)
3. [Phase 3: Incremental Search Narrowing](#phase-3-incremental-search-narrowing)
4. [Phase 4: Typed API Models — Eliminate serde_json::Value](#phase-4-typed-api-models--eliminate-serde_jsonvalue)
5. [Phase 5: Replace Mega-Regex with Lookup Tables](#phase-5-replace-mega-regex-with-lookup-tables)
6. [Phase 6: App Struct Decomposition](#phase-6-app-struct-decomposition)
7. [Phase 7: Index-Based Filtered Views](#phase-7-index-based-filtered-views)
8. [Phase 8: Provider Profile System](#phase-8-provider-profile-system)
9. [Phase 9: Structured Error Handling & Retry Logic](#phase-9-structured-error-handling--retry-logic)
10. [Phase 10: Comprehensive Test Suite](#phase-10-comprehensive-test-suite)

---

## Phase 1: Local Catalog Cache — Instant Cold Starts

### Problem

Every app launch re-fetches the full catalog (categories + all streams for live, VOD, and series) from the provider API. For providers with 30,000+ channels, this takes 5-15 seconds. Users see a loading spinner every time they open the app.

Industry standard (TiviMate, IPTV Smarters Pro) is to cache catalog data locally and load from cache instantly, then refresh in background based on a staleness threshold.

### Goal

Cold start time from app launch to fully navigable UI: **< 200ms** from cache. Background refresh happens silently after UI is rendered.

### Implementation

#### 1.1 Add `bincode` dependency

**File**: `Cargo.toml`

Add under `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`:

```toml
bincode = "1.3"
```

**Why bincode**: Sub-millisecond serialization/deserialization for Rust structs. Benchmarks show bincode is 2-5x faster than MessagePack and 10-50x faster than JSON for structured data. Size is ~60% smaller than JSON. Since this cache is never read by external tools, human-readability is irrelevant.

#### 1.2 Create cache module

**New file**: `src/cache.rs`

```rust
use crate::api::{Category, Stream};
use crate::config::ProcessingMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cache version — increment when CachedCatalog struct changes to auto-invalidate old caches
const CACHE_VERSION: u32 = 1;

/// On-disk catalog cache for a single account
#[derive(Serialize, Deserialize)]
pub struct CachedCatalog {
    pub version: u32,
    pub cached_at: u64,            // Unix timestamp (seconds)
    pub account_name: String,
    pub account_url: String,       // To detect if provider changed

    // Pre-preprocessed data (already filtered by active modes at cache time)
    pub processing_modes: Vec<ProcessingMode>,

    // Live
    pub live_categories: Vec<Category>,
    pub live_streams: Vec<Stream>,

    // VOD
    pub vod_categories: Vec<Category>,
    pub vod_streams: Vec<Stream>,

    // Series
    pub series_categories: Vec<Category>,
    pub series_streams: Vec<Stream>,

    // Metadata
    pub total_channels: usize,
    pub total_movies: usize,
    pub total_series: usize,

    // Category channel counts (category_id -> count)
    pub category_counts: Vec<(String, usize)>,
}

impl CachedCatalog {
    /// Returns the cache file path for a given account.
    /// Path: <config_dir>/cache/<account_name_hash>.bin
    pub fn cache_path(account_name: &str) -> Option<PathBuf> {
        use directories::ProjectDirs;
        let proj = ProjectDirs::from("com", "vibecoding", "vibe-iptv")?;
        let cache_dir = proj.cache_dir().to_path_buf();
        std::fs::create_dir_all(&cache_dir).ok()?;

        // Hash the account name to avoid filesystem issues with special characters
        let hash = simple_hash(account_name);
        Some(cache_dir.join(format!("{}.bin", hash)))
    }

    /// Save catalog to disk. Non-blocking — call from a background task.
    pub fn save(&self) -> Result<(), anyhow::Error> {
        let path = Self::cache_path(&self.account_name)
            .ok_or_else(|| anyhow::anyhow!("Cannot determine cache directory"))?;
        let encoded = bincode::serialize(self)?;
        std::fs::write(&path, encoded)?;
        Ok(())
    }

    /// Load catalog from disk. Returns None if cache doesn't exist, is corrupt, or version mismatches.
    pub fn load(account_name: &str) -> Option<CachedCatalog> {
        let path = Self::cache_path(account_name)?;
        let data = std::fs::read(&path).ok()?;
        let catalog: CachedCatalog = bincode::deserialize(&data).ok()?;

        // Version check — reject outdated cache format
        if catalog.version != CACHE_VERSION {
            let _ = std::fs::remove_file(&path); // Clean up stale cache
            return None;
        }

        Some(catalog)
    }

    /// Check if cache is stale based on auto_refresh_hours setting.
    /// Returns true if cache should be refreshed.
    pub fn is_stale(&self, auto_refresh_hours: u32) -> bool {
        if auto_refresh_hours == 0 {
            return false; // Auto-refresh disabled
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age_hours = (now.saturating_sub(self.cached_at)) / 3600;
        age_hours >= auto_refresh_hours as u64
    }

    /// Check if the active processing modes have changed since cache was built.
    pub fn modes_changed(&self, current_modes: &[ProcessingMode]) -> bool {
        self.processing_modes != current_modes
    }

    /// Delete cache for account
    pub fn invalidate(account_name: &str) {
        if let Some(path) = Self::cache_path(account_name) {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn simple_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
```

#### 1.3 Ensure `Category` and `Stream` are `Serialize + Deserialize` for bincode

**File**: `src/api.rs`

Both `Category` and `Stream` already derive `Serialize, Deserialize`. However, the `#[serde(skip)]` fields (`search_name`, `is_american`, `is_english`, `clean_name`, `cached_parsed`, `latency_ms`, `account_name`) will be zeroed/None on deserialization. This is correct — these are runtime-computed fields.

**Action required**: Ensure `ParsedStream` in `src/parser.rs` does NOT need to be serialized. It is recomputed lazily via `get_or_parse_cached()`. No changes needed.

**Action required**: Confirm `SeriesEpisode`, `ProcessingMode`, and all nested types derive `Serialize, Deserialize`. `ProcessingMode` is in `src/config.rs` and already derives both.

#### 1.4 Integrate cache into the login/load flow

**File**: `src/handlers/async_actions.rs`

**Current flow** (LoginSuccess handler):

1. Set client, user_info, server_info
2. Spawn 3 parallel tasks to fetch categories (live, VOD, series)
3. Each task fetches from API → preprocesses → sends back via channel

**New flow** (LoginSuccess handler):

1. Set client, user_info, server_info
2. Attempt `CachedCatalog::load(account_name)`
3. **If cache hit AND not stale AND modes haven't changed**:
   - Populate all `App` state from cache (categories, streams, counts, etc.)
   - Re-compute `#[serde(skip)]` fields: call `preprocess_categories()` to re-set `search_name`, `is_american`, etc. on each category/stream — OR — also cache these fields (see note below)
   - Navigate to `ContentTypeSelection` immediately
   - Spawn background refresh tasks that will silently update data when complete
   - Set a new `App` field: `background_refresh_active: bool = true`
4. **If cache miss or stale or modes changed**:
   - Fall through to existing fetch logic (no change)

**Cache save trigger**: After `TotalChannelsLoaded`, `TotalMoviesLoaded`, and `TotalSeriesLoaded` have all completed (all 3 background scans done), build a `CachedCatalog` from current `App` state and spawn `tokio::spawn(async { catalog.save() })`.

**Important note on `#[serde(skip)]` fields**: The `search_name`, `clean_name`, `is_american`, `is_english` fields are computed during `preprocess_categories()` and `preprocess_streams()`. Two options:

- **Option A (recommended)**: Remove `#[serde(skip)]` from these fields so they are cached. This avoids re-running preprocessing on cache load. The `cached_parsed` field should remain `#[serde(skip)]` since it's lazily computed.
- **Option B**: Keep `#[serde(skip)]` and re-run preprocessing on cache load. This is simpler but slower.

Choose Option A. Specifically, remove `#[serde(skip)]` from: `search_name`, `is_american`, `is_english`, `clean_name` on both `Category` and `Stream`.

#### 1.5 Cache invalidation triggers

**File**: `src/handlers/async_actions.rs` and `src/handlers/input.rs`

Invalidate cache (call `CachedCatalog::invalidate(account_name)`) when:

- User manually refreshes playlist (presses `r`)
- User changes playlist modes (applies new modes via the mode picker)
- User edits or deletes an account
- User switches accounts (the old account's cache remains valid; just load the new account's cache)

#### 1.6 Background refresh UX

**File**: `src/ui/header.rs`

When `app.background_refresh_active` is true, show a subtle indicator in the header — a small animated dot or text like `syncing...` in dim green. Remove it when background refresh completes.

Do NOT show a loading spinner or block user interaction during background refresh. The user should be able to browse the cached catalog immediately.

#### 1.7 Register the module

**File**: `src/lib.rs`

Add: `pub mod cache;`

#### 1.8 New `App` fields

**File**: `src/app.rs`

Add to `App` struct:

```rust
pub background_refresh_active: bool,
pub cache_loaded: bool, // true if current session loaded from cache
```

Initialize both to `false` in `App::new()`.

---

## Phase 2: List Virtualization Hardening

### Problem

The current windowed rendering in `src/ui/panes.rs` already uses `.skip().take()` to render only visible items. However, there are edge cases:

1. The `ListState` offset is managed manually, which can desync with Ratatui's internal offset tracking
2. The `global_all_streams` list (for "All Channels" category) can hold 30k+ items, and the windowed rendering buffer may still be too large
3. The `render_global_search_pane` function constructs items for all search results

### Goal

Guarantee that at most `viewport_height + scroll_buffer` `ListItem`s are allocated per render frame, regardless of list size. Target: **< 1ms** per list render for 50k items.

### Implementation

#### 2.1 Standardize the windowed rendering pattern

**File**: `src/ui/panes.rs`

Create a shared helper function that all list renderers use:

```rust
/// Calculate the visible window for a list of `total` items.
/// Returns (start, end) indices for the slice to render.
/// `selected` is the currently highlighted index.
/// `viewport_height` is the number of visible rows.
fn visible_window(selected: usize, total: usize, viewport_height: usize) -> (usize, usize) {
    if total == 0 || viewport_height == 0 {
        return (0, 0);
    }
    let half = viewport_height / 2;
    let start = selected.saturating_sub(half);
    let end = (start + viewport_height).min(total);
    // Adjust start if end hit the boundary
    let start = if end == total {
        total.saturating_sub(viewport_height)
    } else {
        start
    };
    (start, end)
}
```

#### 2.2 Apply to all list renderers

**File**: `src/ui/panes.rs`

Refactor `render_categories_pane`, `render_streams_pane`, `render_vod_streams_pane`, `render_series_pane`, `render_episodes_pane`, and `render_global_search_pane` to use `visible_window()`.

**Critical**: When using windowed rendering with Ratatui's `List` widget, the `ListState` offset must be set to the window's `start` index. Otherwise Ratatui will try to scroll internally and fight with the manual windowing.

```rust
let (start, end) = visible_window(selected, total, viewport_height);
let items: Vec<ListItem> = data[start..end]
    .iter()
    .map(|item| /* build ListItem */)
    .collect();

// Sync ListState offset to our window start
list_state.select(Some(selected - start)); // Convert absolute index to window-relative
*list_state.offset_mut() = 0; // Our window IS the viewport — offset is always 0
```

#### 2.3 Apply to global search results

**File**: `src/ui/panes.rs`

The `render_global_search_pane` function currently constructs `ListItem`s for all search results (which can be 100 items — capped by the search logic). This is already bounded and acceptable. No changes needed here, but if the cap is ever raised, apply the same windowed pattern.

#### 2.4 Apply to sports matches and episodes lists

**File**: `src/ui/sports.rs`, `src/ui/series.rs`

Apply the same `visible_window()` pattern to `render_sports_matches_pane` and any episode list rendering.

---

## Phase 3: Incremental Search Narrowing

### Problem

When the user types a search query, the current implementation re-scans the full `all_streams` list on every keystroke using fuzzy matching. For 30k streams, each keystroke triggers 30k `fuzzy_match()` calls.

### Goal

Progressive narrowing: each keystroke filters the _previous result set_, not the full list. Backspace widens back. Target: **< 5ms** per keystroke for 50k streams.

### Implementation

#### 3.1 Add search result stack to SearchState

**File**: `src/errors.rs` (where `SearchState` is defined)

Extend `SearchState`:

```rust
pub struct SearchState {
    pub query: String,
    pub history: VecDeque<String>,
    pub suggestions: Vec<String>,
    pub last_search_time: Option<std::time::Instant>,
    pub debounce_timer: Option<std::time::Instant>,

    // NEW: Incremental narrowing
    /// Stack of (query_length, result_indices) for progressive narrowing.
    /// Each entry records the result set at a given query length.
    /// On backspace, pop to widen. On new char, narrow from top of stack.
    pub narrowing_stack: Vec<(usize, Vec<usize>)>,
}
```

Initialize `narrowing_stack` as `Vec::new()` in `SearchState::default()` / `new()`.

#### 3.2 Implement incremental narrowing logic

**File**: `src/preprocessing.rs` (or a new `src/search.rs` if preferred)

```rust
use crate::api::Stream;
use crate::errors::SearchState;
use std::sync::Arc;

/// Perform incremental search narrowing.
/// Returns indices into `all_streams` that match the current query.
pub fn incremental_search(
    query: &str,
    all_streams: &[Arc<Stream>],
    search_state: &mut SearchState,
    min_score: i64,
) -> Vec<usize> {
    let query_len = query.len();

    if query.is_empty() {
        search_state.narrowing_stack.clear();
        return (0..all_streams.len()).collect();
    }

    // Check if we can narrow from previous results
    if let Some((prev_len, prev_indices)) = search_state.narrowing_stack.last() {
        if query_len > *prev_len && query.starts_with(&search_state.query[..*prev_len]) {
            // User added characters — narrow from previous result set
            let narrowed: Vec<usize> = prev_indices
                .iter()
                .copied()
                .filter(|&idx| all_streams[idx].fuzzy_match(query, min_score))
                .collect();
            search_state.narrowing_stack.push((query_len, narrowed.clone()));
            return narrowed;
        }
    }

    // Check if user deleted characters (backspace) — pop stack
    while let Some((prev_len, _)) = search_state.narrowing_stack.last() {
        if *prev_len >= query_len {
            search_state.narrowing_stack.pop();
        } else {
            break;
        }
    }

    // If stack has a valid base, narrow from it
    if let Some((_, base_indices)) = search_state.narrowing_stack.last() {
        let narrowed: Vec<usize> = base_indices
            .iter()
            .copied()
            .filter(|&idx| all_streams[idx].fuzzy_match(query, min_score))
            .collect();
        search_state.narrowing_stack.push((query_len, narrowed.clone()));
        return narrowed;
    }

    // No valid stack — full scan (first character typed)
    let results: Vec<usize> = all_streams
        .iter()
        .enumerate()
        .filter(|(_, s)| s.fuzzy_match(query, min_score))
        .map(|(idx, _)| idx)
        .collect();
    search_state.narrowing_stack.push((query_len, results.clone()));
    results
}
```

#### 3.3 Integrate into input handlers

**File**: `src/handlers/input.rs`

Wherever the search query is updated (character typed or backspace), replace the current full-list filter call with `incremental_search()`. The returned indices are used to build the display list:

```rust
let matching_indices = incremental_search(
    &app.search_state.query,
    &app.all_streams, // or all_vod_streams, all_series_streams
    &mut app.search_state,
    50, // min_score threshold
);
app.streams = matching_indices
    .iter()
    .map(|&idx| app.all_streams[idx].clone())
    .collect();
```

#### 3.4 Reset stack on context changes

Clear `search_state.narrowing_stack` when:

- User switches categories
- User switches content type (live/VOD/series)
- User exits search mode
- Playlist data is refreshed

---

## Phase 4: Typed API Models — Eliminate serde_json::Value

### Problem

Multiple fields on `Stream`, `UserInfo`, `SeriesInfo`, and `VodInfo` use `serde_json::Value` instead of concrete types. This causes:

- Heap allocations for every `Value` (72 bytes per `Value` enum vs 8 bytes for an `i64`)
- Runtime type checking scattered throughout the codebase (`.as_str()`, `.as_i64()`, `.is_number()`)
- For 30k `Stream` objects, the `stream_id` field alone wastes ~1.3MB of unnecessary heap allocations

The Xtream API is inconsistent across providers (some return `stream_id` as a number, others as a string). This must be handled at the deserialization boundary, not scattered through the codebase.

### Goal

Zero `serde_json::Value` fields on hot-path data structures (`Stream`, `Category`, `UserInfo`). Cold-path structures (`SeriesInfo.episodes`, `VodInfo.info`) may retain `Value` where the schema is genuinely dynamic.

### Implementation

#### 4.1 Create flexible ID type

**File**: `src/api.rs`

```rust
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A stream/category ID that can be deserialized from either a JSON number or string.
/// Stored as a String internally for uniform handling.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FlexId(pub String);

impl FlexId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for FlexId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for FlexId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Number(n) => Ok(FlexId(n.to_string())),
            serde_json::Value::String(s) => Ok(FlexId(s)),
            serde_json::Value::Null => Ok(FlexId(String::new())),
            _ => Ok(FlexId(value.to_string())),
        }
    }
}

impl Serialize for FlexId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}
```

#### 4.2 Create flexible number type

**File**: `src/api.rs`

```rust
/// Deserializes a number that might arrive as a JSON string (e.g., "42" instead of 42).
/// Common in Xtream API responses.
fn deserialize_flex_u64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u64>, D::Error> {
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(n)) => Ok(n.as_u64()),
        Some(serde_json::Value::String(s)) => Ok(s.parse::<u64>().ok()),
        _ => Ok(None),
    }
}

fn deserialize_flex_f32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f32>, D::Error> {
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(n)) => Ok(n.as_f64().map(|f| f as f32)),
        Some(serde_json::Value::String(s)) => Ok(s.parse::<f32>().ok()),
        _ => Ok(None),
    }
}
```

#### 4.3 Update `Stream` struct

**File**: `src/api.rs`

Replace:

```rust
pub num: Option<serde_json::Value>,
#[serde(alias = "series_id", default)]
pub stream_id: serde_json::Value,
pub rating: Option<serde_json::Value>,
pub rating_5: Option<serde_json::Value>,
```

With:

```rust
#[serde(default, deserialize_with = "deserialize_flex_u64")]
pub num: Option<u64>,

#[serde(alias = "series_id", default)]
pub stream_id: FlexId,

#[serde(default, deserialize_with = "deserialize_flex_f32")]
pub rating: Option<f32>,

#[serde(default, deserialize_with = "deserialize_flex_f32")]
pub rating_5: Option<f32>,
```

#### 4.4 Update `Category` struct

**File**: `src/api.rs`

Replace:

```rust
pub parent_id: ::serde_json::Value,
```

With:

```rust
#[serde(default)]
pub parent_id: FlexId,
```

#### 4.5 Update `UserInfo` struct

**File**: `src/api.rs`

Replace all `Option<serde_json::Value>` fields:

```rust
pub struct UserInfo {
    pub auth: i32,
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flex_u64")]
    pub exp_date: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_flex_u64")]
    pub max_connections: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_flex_u64")]
    pub active_cons: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_flex_u64")]
    pub total_live_streams: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_flex_u64")]
    pub total_vod_streams: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_flex_u64")]
    pub total_series_streams: Option<u64>,
}
```

#### 4.6 Update `SeriesEpisode` struct

**File**: `src/api.rs`

Replace:

```rust
pub id: Option<serde_json::Value>,
pub info: Option<serde_json::Value>,
```

With:

```rust
pub id: Option<FlexId>,
pub info: Option<serde_json::Value>, // Keep — genuinely dynamic schema
```

#### 4.7 Update `MovieData` struct

**File**: `src/api.rs`

Replace:

```rust
pub stream_id: serde_json::Value,
pub custom_sid: Option<serde_json::Value>,
```

With:

```rust
pub stream_id: FlexId,
pub custom_sid: Option<FlexId>,
```

#### 4.8 Find and update all call sites

After changing the types above, the compiler will surface every location that uses the old `serde_json::Value` API (`.as_str()`, `.as_i64()`, `.is_number()`, etc.). Replace each with the new typed access:

- `stream.stream_id.as_str().unwrap_or("0")` → `stream.stream_id.as_str()` (already a `&str`)
- Any `get_id_str(&stream)` helper → replace with `stream.stream_id.as_str()`
- `user_info.exp_date.as_i64()` → `user_info.exp_date` (already `Option<u64>`)
- `user_info.total_live_streams.as_str().and_then(...)` → `user_info.total_live_streams` (already `Option<u64>`)

**Search for all call sites**: Run `cargo build` after the struct changes. The compiler errors will identify every location that needs updating. Fix each one. Do NOT use `as serde_json::Value` casts — the goal is to eliminate all `Value` usage on these types.

#### 4.9 Leave dynamic fields as-is

These fields should remain `serde_json::Value` because their schema is genuinely provider-dependent and dynamic:

- `SeriesInfo.episodes` — nested HashMap of season→episodes, structure varies
- `SeriesInfo.seasons` — `Option<Vec<serde_json::Value>>`
- `SeriesInfo.info` — `Option<serde_json::Value>`
- `VodInfo.info` — `Option<serde_json::Value>`
- `SeriesEpisode.info` — `Option<serde_json::Value>`

---

## Phase 5: Replace Mega-Regex with Lookup Tables

### Problem

`src/parser.rs` contains `FOREIGN_PATTERNS_REGEX`, a single compiled regex built by joining 150+ patterns (country codes, language names, region names). This regex is evaluated against every stream/category name during preprocessing. Regex backtracking on this many alternatives is expensive.

Most of these patterns are simple substring matches (`"ARABIC"`, `"TURKISH"`, `"HINDI"`) or prefix/suffix checks (`"AR |"`, `"|AR|"`) that don't need regex.

### Goal

Replace the mega-regex with O(1) HashSet lookups for exact keyword matches and a small, focused regex for structural patterns only. Target: **3-5x speedup** for `is_american_live()` and `is_english_live()` on 30k categories.

### Implementation

#### 5.1 Create keyword lookup tables

**File**: `src/parser.rs`

```rust
use std::collections::HashSet;

/// Country/language keywords that indicate non-American content.
/// Used for O(1) substring matching — no regex needed.
static FOREIGN_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // Middle East & North Africa
        "ARAB", "ARABIC", "SAUDI", "EMIRATES", "QATAR", "KUWAIT", "PERSIAN", "IRAN",
        "AFGHAN", "ISRAEL", "MAROC", "MOROCCO", "TUNISIA", "ALGERIA", "EGYPT",
        // South Asia
        "INDIA", "INDIAN", "HINDI", "PUNJABI", "TAMIL", "TELUGU", "MALAYALAM",
        "KANNADA", "MARATHI", "BENGALI", "PAKISTAN", "URDU", "BANGLA", "BANGLADESH",
        // East Asia
        "CHINA", "CHINESE", "MANDARIN", "CANTONESE", "JAPAN", "KOREA",
        "PHILIPPINES", "FILIPINO", "PINOY", "VIETNAM", "THAILAND", "INDONESIA", "MALAYSIA",
        // Europe (non-English)
        "FRANCE", "FRENCH", "GERMAN", "GERMANY", "DEUTSCH", "ITALY", "ITALIAN",
        "SPAIN", "SPANISH", "ESPANA", "LATINO", "PORTUGAL", "PORTUGUESE", "BRAZIL",
        "DUTCH", "NETHERLANDS", "POLAND", "POLISH", "ROMANIA", "ROMANIAN",
        "CZECH", "HUNGARY", "HUNGARIAN", "GREEK", "GREECE", "ALBANIA", "ALBANIAN",
        "SERBIA", "SERBIAN", "CROATIA", "CROATIAN", "BOSNIA", "BULGARIA", "BULGARIAN",
        "SLOVENIA", "MACEDONIA", "MONTENEGRO", "NORDIC", "SWEDEN", "SWEDISH",
        "NORWAY", "NORWEGIAN", "DENMARK", "DANISH", "FINLAND", "FINNISH",
        "RUSSIA", "RUSSIAN", "UKRAINE", "UKRAINIAN", "BELARUS",
        // Africa
        "AFRICA", "NIGERIA", "KENYA", "SOMALIA", "SOUTH AFRICA",
        // Central Asia / Caucasus
        "TURKEY", "TURK", "ARMENIA", "ARMENIAN", "KURDISH", "KURD",
        "AZERBAIJAN", "GEORGIA", "HONG KONG",
        // UK/Ireland (for 'Merica mode, these are "foreign")
        "UNITED KINGDOM", "BRITISH", "IRELAND", "IRISH", "SCOTLAND",
    ]
    .into_iter()
    .collect()
});

/// Two-letter country code prefixes used in IPTV category names (e.g., "AR |", "FR|").
/// These need structural matching (prefix/suffix with delimiter).
static FOREIGN_COUNTRY_CODES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "AR", "SA", "AE", "QA", "KW", "IR", "AF", "IL", "TR", "IN", "PK", "BD",
        "CN", "JP", "KR", "PH", "VN", "TH", "ID", "MY", "AM", "KH", "AZ", "GE",
        "HK", "ZA", "UK", "IE", "SC", "FR", "DE", "IT", "ES", "PT", "NL", "PL",
        "RO", "CZ", "HU", "GR", "AL", "RS", "HR", "BA", "BG", "SI", "MK", "ME",
        "SE", "NO", "DK", "FI", "RU", "UA", "BY", "BR",
    ]
    .into_iter()
    .collect()
});
```

#### 5.2 Replace `is_american_live` with lookup-based implementation

**File**: `src/parser.rs`

```rust
/// Check if a category name matches foreign (non-American) content.
/// Uses O(1) HashSet lookups instead of regex backtracking.
fn matches_foreign(name: &str) -> bool {
    let upper = name.to_uppercase();

    // 1. Keyword match (O(1) per keyword, checks all keywords)
    for keyword in FOREIGN_KEYWORDS.iter() {
        if upper.contains(keyword) {
            return true;
        }
    }

    // 2. Country code structural match (e.g., "AR |", "|AR|", " AR ")
    for code in FOREIGN_COUNTRY_CODES.iter() {
        // "XX |" or "XX|" prefix
        if upper.starts_with(code) {
            let rest = &upper[code.len()..];
            if rest.starts_with(" |") || rest.starts_with("|") || rest.starts_with(" :") || rest.starts_with(":") {
                return true;
            }
        }
        // "|XX|" infix
        if upper.contains(&format!("|{}|", code)) {
            return true;
        }
        // " XX " standalone word (with word boundaries)
        if upper.contains(&format!(" {} ", code)) {
            return true;
        }
    }

    // 3. "ASIA" special case (standalone word, not "ASIAN" in "EURASIAN" etc.)
    if upper.contains("ASIA") && !upper.contains("EURASIAN") {
        return true;
    }

    false
}

pub fn is_american_live(name: &str) -> bool {
    !matches_foreign(name)
}
```

#### 5.3 Remove the old `FOREIGN_PATTERNS_REGEX`

**File**: `src/parser.rs`

Delete the `FOREIGN_PATTERNS_REGEX` static and all code that references it. Replace all call sites with `matches_foreign()`.

#### 5.4 Apply the same pattern to similar mega-regexes

Review `CLEAN_SUFFIXES` and `CLEAN_BRACKETS_GARBAGE` for candidates. These are smaller (10-20 alternatives) and less critical, but the same HashSet approach can be applied if profiling shows they're hot.

---

## Phase 6: App Struct Decomposition

### Problem

`App` has ~95 fields in a single flat struct. This makes it hard to:

- Understand which fields relate to which feature
- Pass focused state to functions (everything takes `&mut App`)
- Add new features without increasing cognitive load
- Enable contributors to work on isolated subsystems

### Goal

Group fields into focused sub-structs. The `App` struct becomes a thin coordinator. No behavioral changes — purely structural.

### Implementation

#### 6.1 Define sub-state structs

**File**: `src/app.rs`

```rust
/// Content state for a single content type (live, VOD, or series).
/// This struct is generic enough to be reused for all three content types.
pub struct ContentState {
    pub all_categories: Vec<Arc<Category>>,
    pub categories: Vec<Arc<Category>>,       // Filtered/display view
    pub selected_category_index: usize,
    pub category_list_state: ListState,

    pub all_streams: Vec<Arc<Stream>>,
    pub global_all_streams: Vec<Arc<Stream>>, // Full cache for "ALL" category
    pub streams: Vec<Arc<Stream>>,            // Filtered/display view
    pub selected_stream_index: usize,
    pub stream_list_state: ListState,
}

impl ContentState {
    pub fn new() -> Self {
        Self {
            all_categories: Vec::new(),
            categories: Vec::new(),
            selected_category_index: 0,
            category_list_state: ListState::default(),
            all_streams: Vec::new(),
            global_all_streams: Vec::new(),
            streams: Vec::new(),
            selected_stream_index: 0,
            stream_list_state: ListState::default(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Series-specific state (extends ContentState pattern with episodes)
pub struct SeriesState {
    pub content: ContentState,
    pub episodes: Vec<SeriesEpisode>,
    pub selected_episode_index: usize,
    pub episode_list_state: ListState,
    pub current_info: Option<SeriesInfo>,
}

/// Session state for the active provider connection
pub struct SessionState {
    pub client: IptvClient,
    pub user_info: Option<UserInfo>,
    pub server_info: Option<ServerInfo>,
    pub total_channels: usize,
    pub total_movies: usize,
    pub total_series: usize,
    pub provider_timezone: Option<String>,
}

/// UI overlay and transient display state
pub struct UiState {
    pub loading_tick: u64,
    pub loading_progress: Option<LoadingProgress>,
    pub state_loading: bool,
    pub show_help: bool,
    pub show_guide: Option<Guide>,
    pub guide_scroll: u16,
    pub show_play_details: bool,
    pub show_welcome_popup: bool,
    pub pending_play_url: Option<String>,
    pub pending_play_title: Option<String>,
    pub login_error: Option<String>,
    pub player_error: Option<String>,
    pub new_version_available: Option<String>,
    pub background_refresh_active: bool,
    pub cache_loaded: bool,
    // Layout rects for mouse support
    pub area_categories: Rect,
    pub area_streams: Rect,
    pub area_accounts: Rect,
}

/// Sports dashboard state
pub struct SportsState {
    pub matches: Vec<StreamedMatch>,
    pub list_state: ListState,
    pub categories: Vec<String>,
    pub category_list_state: ListState,
    pub selected_category_index: usize,
    pub current_streams: Vec<StreamedStream>,
    pub details_loading: bool,
    pub live_scores: Vec<ScoreGame>,
}

/// Matrix rain animation state
pub struct MatrixRainState {
    pub show: bool,
    pub start_time: Option<Instant>,
    pub screensaver_mode: bool,
    pub columns: Vec<MatrixColumn>,
    pub logo_hits: Vec<bool>,
}

/// Login form state
pub struct LoginFormState {
    pub field_focus: LoginField,
    pub input_name: Input,
    pub input_url: Input,
    pub input_username: Input,
    pub input_password: Input,
    pub input_epg_url: Input,
    pub input_server_timezone: Input,
}

/// Chromecast state
pub struct CastState {
    pub devices: Vec<CastDevice>,
    pub device_list_state: ListState,
    pub show_picker: bool,
    pub discovering: bool,
    pub selected_device_index: usize,
}
```

#### 6.2 Refactor `App` struct

**File**: `src/app.rs`

```rust
pub struct App {
    pub config: AppConfig,
    pub current_screen: CurrentScreen,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub active_pane: Pane,
    pub cached_user_timezone: String,

    // Sub-state groups
    pub session: Option<SessionState>,
    pub live: ContentState,
    pub vod: ContentState,
    pub series: SeriesState,
    pub sports: SportsState,
    pub search: SearchState,
    pub ui: UiState,
    pub login_form: LoginFormState,
    pub matrix_rain: MatrixRainState,
    pub cast: CastState,

    // Account selection (Home screen)
    pub selected_account_index: usize,
    pub account_list_state: ListState,

    // EPG cache (shared across content types)
    pub epg_cache: HashMap<String, String>,
    pub last_focused_stream_id: Option<String>,
    pub focus_timestamp: Option<Instant>,
    pub category_channel_counts: HashMap<String, usize>,

    // VOD info (shared)
    pub current_vod_info: Option<VodInfo>,

    // Settings (Settings screen)
    pub settings_state: SettingsState,
    // ... settings-related list states

    // Groups
    pub selected_group_index: usize,
    pub group_list_state: ListState,
    pub pending_stream_for_group: Option<(String, String)>,
    pub group_name_input: Input,
}
```

#### 6.3 Migrate all field accesses

This is the most labor-intensive step. Every reference to `app.streams` becomes `app.live.streams`. Every reference to `app.vod_categories` becomes `app.vod.categories`.

**Strategy**: Rename fields one sub-struct at a time. After each sub-struct migration, run `cargo build` to find all broken references. Fix them. Repeat.

**Migration order** (each is an independently compilable step):

1. `ContentState` for live (move `all_categories`, `categories`, `all_streams`, `streams`, `global_all_streams`, and their indices/list_states)
2. `ContentState` for VOD (same pattern)
3. `SeriesState` (ContentState + episodes)
4. `SessionState` (client, user_info, server_info, totals)
5. `UiState` (loading, errors, overlays)
6. `SportsState` (sports matches, scores)
7. `MatrixRainState` (animation fields)
8. `LoginFormState` (form inputs)
9. `CastState` (chromecast)

**Important**: Each step is a pure rename/move. No behavioral changes. Tests must pass after each step.

---

## Phase 7: Index-Based Filtered Views

### Problem

The current dual-list pattern (`all_categories` / `categories`) clones `Arc<T>` items into the display list. While `Arc::clone()` is cheap (pointer copy + refcount increment), it still means maintaining two separate vectors that must be kept in sync.

### Goal

Replace the display lists with index vectors (`Vec<usize>`) pointing into the master list. This eliminates the sync problem and reduces memory for the filtered view to a compact index array.

### Implementation

#### 7.1 Update `ContentState`

**File**: `src/app.rs`

```rust
pub struct ContentState {
    // Master data (immutable after load)
    pub all_categories: Vec<Category>,        // Owned, no Arc
    pub all_streams: Vec<Stream>,             // Owned, no Arc
    pub global_all_streams: Vec<Stream>,      // Full cache for "ALL" category

    // Filtered views (indices into master data)
    pub category_view: Vec<usize>,            // Indices into all_categories
    pub stream_view: Vec<usize>,              // Indices into all_streams

    // Selection state
    pub selected_category_index: usize,       // Index into category_view
    pub selected_stream_index: usize,         // Index into stream_view
    pub category_list_state: ListState,
    pub stream_list_state: ListState,
}

impl ContentState {
    /// Get the currently selected category, if any.
    pub fn selected_category(&self) -> Option<&Category> {
        self.category_view
            .get(self.selected_category_index)
            .and_then(|&idx| self.all_categories.get(idx))
    }

    /// Get the currently selected stream, if any.
    pub fn selected_stream(&self) -> Option<&Stream> {
        self.stream_view
            .get(self.selected_stream_index)
            .and_then(|&idx| self.all_streams.get(idx))
    }

    /// Get a mutable reference to the currently selected stream.
    pub fn selected_stream_mut(&mut self) -> Option<&mut Stream> {
        self.stream_view
            .get(self.selected_stream_index)
            .copied()
            .and_then(move |idx| self.all_streams.get_mut(idx))
    }

    /// Rebuild the category view with a filter predicate.
    pub fn filter_categories(&mut self, predicate: impl Fn(&Category) -> bool) {
        self.category_view = self.all_categories
            .iter()
            .enumerate()
            .filter(|(_, cat)| predicate(cat))
            .map(|(idx, _)| idx)
            .collect();
        self.selected_category_index = 0;
    }

    /// Rebuild the stream view with a filter predicate.
    pub fn filter_streams(&mut self, predicate: impl Fn(&Stream) -> bool) {
        self.stream_view = self.all_streams
            .iter()
            .enumerate()
            .filter(|(_, stream)| predicate(stream))
            .map(|(idx, _)| idx)
            .collect();
        self.selected_stream_index = 0;
    }

    /// Show all categories (reset filter).
    pub fn show_all_categories(&mut self) {
        self.category_view = (0..self.all_categories.len()).collect();
    }

    /// Show all streams (reset filter).
    pub fn show_all_streams(&mut self) {
        self.stream_view = (0..self.all_streams.len()).collect();
    }

    /// Get stream at view index for rendering.
    pub fn view_stream(&self, view_idx: usize) -> Option<&Stream> {
        self.stream_view
            .get(view_idx)
            .and_then(|&idx| self.all_streams.get(idx))
    }

    /// Number of items in the filtered view.
    pub fn category_count(&self) -> usize {
        self.category_view.len()
    }

    pub fn stream_count(&self) -> usize {
        self.stream_view.len()
    }
}
```

#### 7.2 Update UI rendering to use views

**File**: `src/ui/panes.rs`

Replace:

```rust
app.streams.iter().enumerate().skip(start).take(count)
```

With:

```rust
app.live.stream_view[start..end].iter().map(|&idx| &app.live.all_streams[idx])
```

#### 7.3 Remove `Arc` wrapper

Since streams are now owned by `ContentState` and views are indices, `Arc<Category>` and `Arc<Stream>` are no longer needed. Replace `Vec<Arc<Category>>` with `Vec<Category>` and `Vec<Arc<Stream>>` with `Vec<Stream>` throughout.

**Exception**: If any async task needs to hold a reference to a stream across an await point, it should clone the `Stream` at that point rather than holding an `Arc`. This is fine because it's a one-off clone for a specific operation, not a systemic pattern.

---

## Phase 8: Provider Profile System

### Problem

`src/preprocessing.rs` and `src/parser.rs` contain hardcoded logic for specific IPTV providers ("strong", "trex", "mega"). Example from `preprocess_categories()`:

```rust
if account_lower.contains("strong") {
    let name = c.category_name.to_uppercase();
    if name.starts_with("AR |") || name.starts_with("AR|") || name.starts_with("AR :") {
        c.is_american = false;
    }
    if name.contains("NBA PASS") || name.contains("NBA REAL") || name.contains("NHL REAL") {
        c.is_american = false;
    }
}
```

This is brittle, hard to extend, and makes the codebase provider-dependent.

### Goal

Move provider-specific rules into a data-driven configuration system. Users can define their own profiles. Ship defaults for known providers. New providers require zero code changes.

### Implementation

#### 8.1 Define provider profile types

**File**: `src/config.rs`

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderProfile {
    /// Regex pattern to match against account name (case-insensitive).
    /// If this matches, the profile's rules apply.
    pub account_pattern: String,

    /// Category name prefixes that should be marked as non-American.
    /// Checked case-insensitively.
    pub excluded_prefixes: Vec<String>,

    /// Category name substrings that should be marked as non-American.
    /// Checked case-insensitively.
    pub excluded_keywords: Vec<String>,
}

impl Default for ProviderProfile {
    fn default() -> Self {
        Self {
            account_pattern: String::new(),
            excluded_prefixes: Vec::new(),
            excluded_keywords: Vec::new(),
        }
    }
}
```

#### 8.2 Ship default profiles

**File**: `src/config.rs`

```rust
pub fn default_provider_profiles() -> Vec<ProviderProfile> {
    vec![
        ProviderProfile {
            account_pattern: "(?i)strong".to_string(),
            excluded_prefixes: vec!["AR |".into(), "AR|".into(), "AR :".into()],
            excluded_keywords: vec!["NBA PASS".into(), "NBA REAL".into(), "NHL REAL".into()],
        },
        ProviderProfile {
            account_pattern: "(?i)trex".to_string(),
            excluded_prefixes: vec![],
            excluded_keywords: vec!["NBA NETWORK".into(), "NBA LEAGUE PASS".into()],
        },
    ]
}
```

#### 8.3 Add profiles to `AppConfig`

**File**: `src/config.rs`

Add to `AppConfig`:

```rust
#[serde(default = "default_provider_profiles")]
pub provider_profiles: Vec<ProviderProfile>,
```

#### 8.4 Use profiles in preprocessing

**File**: `src/preprocessing.rs`

Replace the hardcoded `if account_lower.contains("strong")` blocks with:

```rust
// Find matching provider profile
let matching_profile = app_config.provider_profiles.iter().find(|p| {
    Regex::new(&p.account_pattern)
        .map(|re| re.is_match(account_name))
        .unwrap_or(false)
});

if let Some(profile) = matching_profile {
    let upper_name = c.category_name.to_uppercase();
    for prefix in &profile.excluded_prefixes {
        if upper_name.starts_with(&prefix.to_uppercase()) {
            c.is_american = false;
            break;
        }
    }
    for keyword in &profile.excluded_keywords {
        if upper_name.contains(&keyword.to_uppercase()) {
            c.is_american = false;
            break;
        }
    }
}
```

#### 8.5 Cache compiled regexes for profiles

To avoid recompiling the profile's `account_pattern` regex on every category, compile them once at startup and store alongside the config:

```rust
pub struct CompiledProfile {
    pub pattern: Regex,
    pub excluded_prefixes: Vec<String>,
    pub excluded_keywords: Vec<String>,
}
```

Build `Vec<CompiledProfile>` from `AppConfig.provider_profiles` in `App::new()` and store on `App`.

---

## Phase 9: Structured Error Handling & Retry Logic

### Problem

Errors are stored as `Option<String>` and displayed as plain text overlays. There is no retry logic for transient network failures. IPTV providers have notoriously unreliable servers — a single failed request should not halt the user's session.

The existing `IptvError` enum in `src/errors.rs` is well-designed but is not used consistently throughout the codebase. Most error handling uses `anyhow::Error` with ad-hoc string messages.

### Goal

- Use `IptvError` consistently for all API/network errors
- Add automatic retry for retryable errors (network timeouts, 5xx responses)
- Show user-friendly error messages with actionable diagnostics
- Never block the UI for a retryable error

### Implementation

#### 9.1 Extend `IptvError` for retry semantics

**File**: `src/errors.rs`

Add a method to `IptvError`:

```rust
impl IptvError {
    /// Whether this error is transient and should be retried.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            IptvError::DnsResolution(_, _)
                | IptvError::ConnectionTimeout(_, _)
                | IptvError::ServerError(status, _) if *status >= 500
        )
    }

    /// Suggested delay before retry (exponential backoff base).
    pub fn retry_delay_ms(&self) -> u64 {
        match self {
            IptvError::ConnectionTimeout(_, _) => 2000,
            IptvError::ServerError(_, _) => 1000,
            IptvError::DnsResolution(_, _) => 3000,
            _ => 1000,
        }
    }
}
```

#### 9.2 Add retry wrapper for API calls

**File**: `src/api.rs`

```rust
/// Retry an async operation with exponential backoff.
/// Returns the result of the first successful attempt, or the last error.
pub async fn with_retry<F, Fut, T>(
    max_attempts: u32,
    base_delay_ms: u64,
    operation: F,
) -> Result<T, anyhow::Error>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
{
    let mut last_error = None;
    for attempt in 0..max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Check if error is retryable
                let retryable = e.downcast_ref::<IptvError>()
                    .map(|ie| ie.is_retryable())
                    .unwrap_or(false);

                if !retryable || attempt == max_attempts - 1 {
                    return Err(e);
                }

                let delay = base_delay_ms * 2u64.pow(attempt);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                last_error = Some(e);
            }
        }
    }
    Err(last_error.unwrap())
}
```

#### 9.3 Apply retry to critical API calls

**File**: `src/handlers/async_actions.rs`

Wrap category and stream fetch operations:

```rust
// Before:
let cats = client.get_live_categories().await?;

// After:
let cats = crate::api::with_retry(3, 1000, || {
    let client = client.clone();
    async move { client.get_live_categories().await }
}).await?;
```

Apply to: `get_live_categories`, `get_live_streams`, `get_vod_categories`, `get_vod_streams`, `get_series_categories`, `get_series`, `authenticate`, `get_series_info`, `get_vod_info`.

Do NOT apply retry to: stream health checks (these are informational), EPG fetches (debounced, low priority), score fetches (periodic, will retry on next interval).

#### 9.4 Replace string errors with `AppError` in UI state

**File**: `src/app.rs` (or `src/errors.rs`)

```rust
#[derive(Debug, Clone)]
pub enum AppError {
    Network {
        message: String,
        retryable: bool,
        diagnostics: Option<String>,
    },
    Auth {
        message: String,
    },
    Player {
        message: String,
        log_hint: Option<String>,
    },
    StreamDead {
        stream_name: String,
        status: Option<u16>,
    },
}

impl AppError {
    pub fn display_message(&self) -> &str {
        match self {
            AppError::Network { message, .. } => message,
            AppError::Auth { message } => message,
            AppError::Player { message, .. } => message,
            AppError::StreamDead { stream_name, .. } => stream_name,
        }
    }
}
```

Replace `login_error: Option<String>` and `player_error: Option<String>` with:

```rust
pub error: Option<AppError>,
```

Update all error display sites in `src/ui/popups.rs` and `src/ui/common.rs` to use `AppError::display_message()` and optionally show diagnostics.

---

## Phase 10: Comprehensive Test Suite

### Problem

The codebase has 3 integration tests and no unit tests for business logic. The parser (`src/parser.rs`) has 20+ pre-compiled regexes and complex conditional logic. The preprocessing (`src/preprocessing.rs`) has provider-specific rules and mode interactions. These are the highest-risk modules for regressions.

### Goal

Unit test coverage for all pure-logic modules. Integration test coverage for the cache and async action flows.

### Implementation

#### 10.1 Parser tests

**File**: `tests/parser_tests.rs` (new file)

Test every public function in `src/parser.rs`:

```rust
use matrix_iptv_lib::parser::*;

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_stream ---

    #[test]
    fn parse_stream_plain_channel_name() {
        let result = parse_stream("ESPN HD", None);
        assert_eq!(result.display_name, "ESPN HD");
        assert!(!result.is_sports_event);
    }

    #[test]
    fn parse_stream_sports_event_with_time() {
        let result = parse_stream("NBA: Lakers vs Celtics 7:30 PM ET", None);
        assert!(result.is_sports_event);
        assert!(result.display_name.contains("Lakers"));
        assert!(result.display_name.contains("Celtics"));
    }

    #[test]
    fn parse_stream_movie_with_year() {
        let result = parse_stream("The Matrix (1999)", None);
        assert_eq!(result.year, Some(1999));
    }

    #[test]
    fn parse_stream_with_provider_timezone() {
        let result = parse_stream("Match 3:00 PM EST", Some("+05:00"));
        // Should convert time based on provider timezone
        assert!(result.display_name.contains("Match"));
    }

    // --- is_american_live ---

    #[test]
    fn is_american_espn() {
        assert!(is_american_live("USA | ESPN HD"));
        assert!(is_american_live("ESPN"));
        assert!(is_american_live("US Sports"));
    }

    #[test]
    fn is_not_american_arabic() {
        assert!(!is_american_live("AR | MBC Drama"));
        assert!(!is_american_live("Arabic Movies"));
    }

    #[test]
    fn is_not_american_french() {
        assert!(!is_american_live("FR | Canal+"));
        assert!(!is_american_live("France 24"));
    }

    #[test]
    fn is_american_edge_cases() {
        // "AREA" contains "AR" but should NOT be filtered
        assert!(is_american_live("Bay Area Sports"));
        // "HEART" contains "AR" but should NOT be filtered
        assert!(is_american_live("Heart FM"));
        // "PARSING" contains "AR" but should NOT be filtered
        assert!(is_american_live("Sports Parsing Channel"));
    }

    // --- is_english_live ---

    #[test]
    fn is_english_basic() {
        assert!(is_english_live("UK | BBC One"));
        assert!(is_english_live("CA | TSN"));
        assert!(is_english_live("US | Fox News"));
    }

    #[test]
    fn is_not_english_spanish() {
        assert!(!is_english_live("ES | Telecinco"));
    }

    // --- clean_american_name ---

    #[test]
    fn clean_removes_country_prefix() {
        assert_eq!(clean_american_name("USA | ESPN"), "ESPN");
        assert_eq!(clean_american_name("US: Fox News"), "Fox News");
        assert_eq!(clean_american_name("UNITED STATES - CNN"), "CNN");
    }

    #[test]
    fn clean_removes_country_suffix() {
        assert_eq!(clean_american_name("ESPN (USA)"), "ESPN");
        assert_eq!(clean_american_name("Fox News [US]"), "Fox News");
    }

    #[test]
    fn clean_preserves_meaningful_content() {
        // Should not mangle names that happen to contain country codes
        let result = clean_american_name("USA Network");
        assert!(!result.is_empty());
    }

    // --- is_sports_content ---

    #[test]
    fn detects_sports_categories() {
        assert!(is_sports_content("NBA Basketball"));
        assert!(is_sports_content("NFL Football"));
        assert!(is_sports_content("Sports HD"));
        assert!(is_sports_content("UFC Fight Night"));
    }

    #[test]
    fn rejects_non_sports() {
        assert!(!is_sports_content("HBO Movies"));
        assert!(!is_sports_content("Discovery Channel"));
    }
}
```

#### 10.2 Preprocessing tests

**File**: `tests/preprocessing_tests.rs` (new file)

```rust
use matrix_iptv_lib::api::Category;
use matrix_iptv_lib::config::ProcessingMode;
use matrix_iptv_lib::preprocessing::*;
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_categories(names: &[&str]) -> Vec<Category> {
        names.iter().map(|name| Category {
            category_id: name.to_string(),
            category_name: name.to_string(),
            ..Default::default()
        }).collect()
    }

    #[test]
    fn merica_mode_filters_foreign_categories() {
        let mut cats = make_categories(&[
            "USA | Sports", "AR | MBC", "FR | Canal+", "ESPN HD", "DE | ZDF"
        ]);
        let favorites = HashSet::new();
        preprocess_categories(&mut cats, &favorites, &[ProcessingMode::Merica], true, false, "test");

        let names: Vec<&str> = cats.iter().map(|c| c.category_name.as_str()).collect();
        // Should keep American/English, filter out Arabic, French, German
        assert!(names.iter().any(|n| n.contains("ESPN") || n.contains("Sports")));
        assert!(!names.iter().any(|n| n.contains("MBC") || n.contains("Canal") || n.contains("ZDF")));
    }

    #[test]
    fn sports_mode_filters_non_sports() {
        let mut cats = make_categories(&[
            "NBA Basketball", "HBO Movies", "NFL Football", "Discovery"
        ]);
        let favorites = HashSet::new();
        preprocess_categories(&mut cats, &favorites, &[ProcessingMode::Sports], true, false, "test");

        let names: Vec<&str> = cats.iter().map(|c| c.category_name.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("NBA")));
        assert!(names.iter().any(|n| n.contains("NFL")));
        assert!(!names.iter().any(|n| n.contains("HBO")));
        assert!(!names.iter().any(|n| n.contains("Discovery")));
    }

    #[test]
    fn all_english_mode_keeps_english_content() {
        let mut cats = make_categories(&[
            "UK | BBC", "CA | TSN", "FR | TF1", "US | ESPN"
        ]);
        let favorites = HashSet::new();
        preprocess_categories(&mut cats, &favorites, &[ProcessingMode::AllEnglish], true, false, "test");

        let names: Vec<&str> = cats.iter().map(|c| c.category_name.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("BBC")));
        assert!(!names.iter().any(|n| n.contains("TF1")));
    }

    #[test]
    fn combined_modes_are_additive() {
        let mut cats = make_categories(&[
            "USA | NBA", "USA | HBO", "UK | Premier League", "FR | Ligue 1"
        ]);
        let favorites = HashSet::new();
        preprocess_categories(
            &mut cats, &favorites,
            &[ProcessingMode::Merica, ProcessingMode::Sports],
            true, false, "test"
        );

        let names: Vec<&str> = cats.iter().map(|c| c.category_name.as_str()).collect();
        // Should keep American Sports, filter American non-sports AND foreign sports
        assert!(names.iter().any(|n| n.contains("NBA")));
        // HBO is American but not Sports — should be filtered by Sports mode
        assert!(!names.iter().any(|n| n.contains("HBO")));
    }

    #[test]
    fn all_category_always_preserved() {
        let mut cats = make_categories(&["FR | TF1", "DE | ZDF"]);
        let favorites = HashSet::new();
        preprocess_categories(&mut cats, &favorites, &[ProcessingMode::Merica], true, false, "test");

        // "ALL" should be injected even if everything else is filtered
        assert!(cats.iter().any(|c| c.category_id == "ALL"));
    }
}
```

#### 10.3 Cache tests

**File**: `tests/cache_tests.rs` (new file)

```rust
use matrix_iptv_lib::cache::CachedCatalog;
use matrix_iptv_lib::api::{Category, Stream};
use matrix_iptv_lib::config::ProcessingMode;

#[test]
fn cache_roundtrip() {
    let catalog = CachedCatalog {
        version: 1,
        cached_at: 1700000000,
        account_name: "test_account".into(),
        account_url: "http://test.com".into(),
        processing_modes: vec![ProcessingMode::Merica],
        live_categories: vec![Category {
            category_id: "1".into(),
            category_name: "Sports".into(),
            ..Default::default()
        }],
        live_streams: vec![],
        vod_categories: vec![],
        vod_streams: vec![],
        series_categories: vec![],
        series_streams: vec![],
        total_channels: 100,
        total_movies: 50,
        total_series: 25,
        category_counts: vec![("1".into(), 100)],
    };

    // Save
    catalog.save().expect("Cache save failed");

    // Load
    let loaded = CachedCatalog::load("test_account").expect("Cache load failed");
    assert_eq!(loaded.account_name, "test_account");
    assert_eq!(loaded.live_categories.len(), 1);
    assert_eq!(loaded.total_channels, 100);

    // Cleanup
    CachedCatalog::invalidate("test_account");
    assert!(CachedCatalog::load("test_account").is_none());
}

#[test]
fn cache_staleness_detection() {
    let catalog = CachedCatalog {
        version: 1,
        cached_at: 0, // Unix epoch — very old
        account_name: "stale_test".into(),
        account_url: "http://test.com".into(),
        processing_modes: vec![],
        live_categories: vec![],
        live_streams: vec![],
        vod_categories: vec![],
        vod_streams: vec![],
        series_categories: vec![],
        series_streams: vec![],
        total_channels: 0,
        total_movies: 0,
        total_series: 0,
        category_counts: vec![],
    };

    assert!(catalog.is_stale(12)); // 12 hours — epoch is definitely stale
    assert!(!catalog.is_stale(0)); // 0 = disabled
}

#[test]
fn cache_mode_change_detection() {
    let catalog = CachedCatalog {
        version: 1,
        cached_at: 0,
        account_name: "mode_test".into(),
        account_url: "http://test.com".into(),
        processing_modes: vec![ProcessingMode::Merica],
        live_categories: vec![],
        live_streams: vec![],
        vod_categories: vec![],
        vod_streams: vec![],
        series_categories: vec![],
        series_streams: vec![],
        total_channels: 0,
        total_movies: 0,
        total_series: 0,
        category_counts: vec![],
    };

    assert!(!catalog.modes_changed(&[ProcessingMode::Merica]));
    assert!(catalog.modes_changed(&[ProcessingMode::Sports]));
    assert!(catalog.modes_changed(&[ProcessingMode::Merica, ProcessingMode::Sports]));
}
```

#### 10.4 FlexId deserialization tests

**File**: `tests/api_types_tests.rs` (new file)

```rust
use matrix_iptv_lib::api::FlexId;

#[test]
fn flex_id_from_number() {
    let json = r#"42"#;
    let id: FlexId = serde_json::from_str(json).unwrap();
    assert_eq!(id.as_str(), "42");
}

#[test]
fn flex_id_from_string() {
    let json = r#""abc123""#;
    let id: FlexId = serde_json::from_str(json).unwrap();
    assert_eq!(id.as_str(), "abc123");
}

#[test]
fn flex_id_from_null() {
    let json = r#"null"#;
    let id: FlexId = serde_json::from_str(json).unwrap();
    assert!(id.is_empty());
}

#[test]
fn stream_deserialize_mixed_types() {
    // Simulates real Xtream API response with inconsistent types
    let json = r#"{
        "num": "1",
        "name": "ESPN HD",
        "stream_type": "live",
        "stream_id": 12345,
        "category_id": "3",
        "rating": "4.5",
        "rating_5": 3
    }"#;
    let stream: matrix_iptv_lib::api::Stream = serde_json::from_str(json).unwrap();
    assert_eq!(stream.stream_id.as_str(), "12345");
    assert_eq!(stream.num, Some(1));
    assert!((stream.rating.unwrap() - 4.5).abs() < 0.01);
    assert!((stream.rating_5.unwrap() - 3.0).abs() < 0.01);
}
```

#### 10.5 Run all tests

After implementing each test file, verify:

```bash
cargo test                           # All tests pass
cargo test parser_tests              # Parser tests
cargo test preprocessing_tests      # Preprocessing tests
cargo test cache_tests              # Cache tests
cargo test api_types_tests          # API type tests
```

---

## Appendix A: IPTV Best Practices Applied

This spec incorporates industry best practices from leading IPTV applications:

| Practice                                                                      | Implementation                                                                         | Source                                                                                                                              |
| ----------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| **Local EPG caching** — only download updates when stale                      | Phase 1: Binary catalog cache with staleness detection                                 | [IPTV Content Caching Strategies](https://www.vucos.io/post/complete-guide-to-iptv-content-caching-maximizing-delivery-performance) |
| **Hierarchical caching** — edge cache (local) before origin (API)             | Phase 1: Load from local bincode cache, refresh from API in background                 | [IPTV Content Caching Strategies](https://www.vucos.io/post/complete-guide-to-iptv-content-caching-maximizing-delivery-performance) |
| **90-second abandonment threshold** — users leave after 90s of buffering      | Phase 1: Instant UI from cache eliminates all cold-start waiting                       | [IPTV Content Caching Strategies](https://www.vucos.io/post/complete-guide-to-iptv-content-caching-maximizing-delivery-performance) |
| **Daily EPG refresh** — refresh on configurable schedule, not every launch    | Phase 1: `auto_refresh_hours` (default 12h) with background sync                       | [EPG Setup Guide](https://purplecrystal.net/how-epg-in-iptv-works/)                                                                 |
| **Adaptive bitrate and cache management** (TiviMate 2026)                     | Phase 2: Virtualized list rendering; only allocate visible items                       | [TiviMate vs IPTV Smarters](https://www.the-best-iptv.com/iptv-smarters-pro-vs-tivimate/)                                           |
| **Multiple playlist/subscription management**                                 | Already implemented (multi-account); Phase 8 adds per-provider profiles                | [IPTV Smarters Guide](https://courses.specialchem.com/blogs/news/iptv-smarters-pro-the-ultimate-guide-for-seamless-streaming)       |
| **Xtream API inconsistency handling** — providers return different JSON types | Phase 4: `FlexId` and `deserialize_flex_*` handle number/string/null at parse boundary | [Xtream Codes API](https://github.com/zaclimon/xipl/wiki/Xtream-Codes-API)                                                          |
| **Retry with backoff for transient failures**                                 | Phase 9: `with_retry()` for all API calls with exponential backoff                     | IPTV servers are notoriously unreliable                                                                                             |
| **Stream health pre-flight checks**                                           | Already implemented in `player.rs`; Phase 9 adds structured error types                | [Xtream Codes API health endpoints](https://github.com/topics/xtream-codes)                                                         |

## Appendix B: Expected Performance Targets

| Metric                        | Current           | Target                   | Phase |
| ----------------------------- | ----------------- | ------------------------ | ----- |
| Cold start (cached)           | 5-15s             | < 200ms                  | 1     |
| Cold start (first launch)     | 5-15s             | 5-15s (unchanged)        | —     |
| List render (30k items)       | ~5ms              | < 1ms                    | 2     |
| Search keystroke latency      | ~30ms (full scan) | < 5ms (incremental)      | 3     |
| Memory per Stream object      | ~400 bytes        | ~200 bytes               | 4, 7  |
| `is_american_live()` per call | ~50μs (regex)     | ~5μs (HashSet)           | 5     |
| Category filter (mode change) | Clone 30k Arcs    | Rebuild index Vec<usize> | 7     |
| Network error recovery        | Manual refresh    | Auto-retry (3 attempts)  | 9     |

## Appendix C: New Dependencies

| Crate     | Version | Purpose                                | Size Impact |
| --------- | ------- | -------------------------------------- | ----------- |
| `bincode` | 1.3     | Binary serialization for catalog cache | ~30KB       |

No other new dependencies. All other changes use existing crates (`serde`, `regex`, `tokio`, `rayon`, `once_cell`, `fuzzy-matcher`).

## Appendix D: Files Modified Per Phase

| Phase | Files Modified                                                                                                                       | Files Created                                                                                               |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------- |
| 1     | `Cargo.toml`, `src/lib.rs`, `src/app.rs`, `src/api.rs`, `src/handlers/async_actions.rs`, `src/handlers/input.rs`, `src/ui/header.rs` | `src/cache.rs`                                                                                              |
| 2     | `src/ui/panes.rs`, `src/ui/sports.rs`, `src/ui/series.rs`                                                                            | —                                                                                                           |
| 3     | `src/errors.rs`, `src/preprocessing.rs`, `src/handlers/input.rs`                                                                     | —                                                                                                           |
| 4     | `src/api.rs`, all files referencing `stream_id`, `rating`, `num`, `exp_date`, etc.                                                   | —                                                                                                           |
| 5     | `src/parser.rs`, `src/preprocessing.rs`                                                                                              | —                                                                                                           |
| 6     | `src/app.rs`, all files referencing `app.*` fields (every handler, every UI module)                                                  | —                                                                                                           |
| 7     | `src/app.rs`, `src/ui/panes.rs`, `src/handlers/async_actions.rs`, `src/preprocessing.rs`                                             | —                                                                                                           |
| 8     | `src/config.rs`, `src/preprocessing.rs`, `src/app.rs`                                                                                | —                                                                                                           |
| 9     | `src/errors.rs`, `src/api.rs`, `src/app.rs`, `src/handlers/async_actions.rs`, `src/ui/popups.rs`                                     | —                                                                                                           |
| 10    | —                                                                                                                                    | `tests/parser_tests.rs`, `tests/preprocessing_tests.rs`, `tests/cache_tests.rs`, `tests/api_types_tests.rs` |

## Appendix E: Execution Checklist

For each phase:

- [ ] Read this spec's phase section completely before starting
- [ ] Create a git branch: `enhancement/phase-N-description`
- [ ] Implement changes as described
- [ ] Run `cargo check` — zero errors
- [ ] Run `cargo build` — zero warnings (or document why a warning is acceptable)
- [ ] Run `cargo test` — all tests pass
- [ ] Run `cargo fmt` — code is formatted
- [ ] Verify no `serde_json::Value` regressions (Phase 4+): `grep -r "serde_json::Value" src/api.rs` should show only the sanctioned fields
- [ ] Verify no real credentials in any file
- [ ] Commit with descriptive message referencing the phase number
