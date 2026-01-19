# IPTV App Changes Summary

## Date: 2026-01-18 (v3.3.1)

### Changes Implemented

#### ✅ 1. Restored Detailed Help Guides

- Re-implemented the technical and instructional depth for "The TUI Edge", "Playlist Acquisition", and "IPTV Protocol".
- Guides now feature rich formatting and specific advice (Marketplaces, Providers, Technical Specs).

#### ✅ 2. UI/UX Refinement

- Moved version identifier to the bottom-left of the footer for better visibility during testing.
- Updated Home Screen labels to be more descriptive ("The TUI Edge", "Understanding the Protocol").
- Fixed the "Gray Icon" issue caused by leftover sports league fallbacks.
- Simplified Category view to use a clean folder icon system.

#### ✅ 3. Improved MPV Isolation

- Detached MPV from terminal input/output to prevent keyboard shortcut conflicts.
- On Windows, MPV now runs in a detached process to ensure CLI focus remains on the app.

#### ✅ 4. Infrastructure & Test Sync

- Updated global `test-matrix-iptv` command and PowerShell profile to point to the current (OneDrive) workspace.
- Fixed a version mismatch where tests were running older cached binaries.
- Established a new `update_and_test` workflow to ensure builds are synced to the Node wrapper's `bin/` directory.

---

## Date: 2025-12-27 (v3.0.8)

### Changes Implemented

#### ✅ 1. Fixed Critical Out-of-Bounds Panic

**Issue:** Application crashed with `index outside of buffer` error on Mac/Windows when the terminal window was small (e.g., 80 columns).
**Files Modified:**

- `src/matrix_rain.rs`
- `src/ui/loading.rs`
- `src/ui/groups.rs`

**Changes:**

- Updated Matrix logo width to correct value (103).
- Added defensive bounds checking and clipping to all manual coordinate drawing.
- Clamped popup heights to terminal height.
- Implemented `intersection` clipping for large UI elements.

**Rationale:** Ensures application stability across all terminal sizes and operating systems.

---

## Date: 2025-12-21

### Changes Implemented

#### ✅ 1. Renamed "ARCHIVE_DATA" to "MOVIE_DATA"

**Files Modified:**

- `src/ui.rs`

**Changes:**

- **Line 167-169**: Changed header tab from `ARCHIVE_ACCESS` to `MOVIE_ACCESS`
- **Line 799-801**: Changed VOD streams pane title from `ARCHIVE_DATA` to `MOVIE_DATA`

**Rationale:** Better reflects the content type (movies/VOD) instead of generic "archive" terminology.

---

#### ✅ 2. Added "All Movies" Category for VOD

**Files Modified:**

- `src/main.rs`

**Changes:**

- **Line 237-253**: Injected "All Movies" category at index 0 of VOD categories list
  - Category ID: "ALL"
  - Category Name: "All Movies"
  - Similar to how "All Channels" works for live TV

**Rationale:** Allows users to browse all VOD content across all categories without having to select individual categories.

---

#### ✅ 3. Implemented "All Movies" Stream Loading

**Files Modified:**

- `src/main.rs`

**Changes:**

- **Line 1362-1403**: Modified VOD stream loading logic to handle "ALL" category
  - When "ALL" category is selected, calls `client.get_vod_streams_all().await`
  - Otherwise, calls `client.get_vod_streams(&cat_id).await` for specific category
  - Properly formatted with consistent indentation and error handling

**Code Structure:**

```rust
tokio::spawn(async move {
    // Handle "All Movies" category
    if cat_id == "ALL" {
        match client.get_vod_streams_all().await {
            Ok(streams) => { /* Load all streams */ }
            Err(e) => { /* Handle error */ }
        }
    } else {
        match client.get_vod_streams(&cat_id).await {
            Ok(streams) => { /* Load category streams */ }
            Err(e) => { /* Handle error */ }
        }
    }
});
```

---

### Performance Considerations

**Existing Optimizations:**

- Windowed rendering is already implemented in `src/ui.rs` (lines 608-702)
- Only visible items are parsed and rendered, not the entire list
- This should provide good scrolling performance even with large datasets

**Potential Performance Issues:**

1. Network latency when loading large "All Movies" list
2. Parsing overhead in `parse_movie()` and `parse_stream()` functions
3. The windowed rendering uses a half-window buffer which should be optimal

**Recommendation:**

- Monitor performance with real-world data
- If issues persist, consider:
  - Caching parsed results
  - Implementing lazy loading/pagination
  - Optimizing the parsing functions

---

### Testing

**Status:** ✅ App compiles and runs successfully

**Test Command:**

```bash
cargo run --bin matrix-iptv
```

**Expected Behavior:**

1. Navigate to VOD section (press 'v' from live channels)
2. "All Movies" should appear as the first category
3. Selecting "All Movies" should load all VOD streams across all categories
4. Header should show "MOVIE_ACCESS" instead of "ARCHIVE_ACCESS"
5. Stream list should show "MOVIE_DATA" instead of "ARCHIVE_DATA"

---

### Files Created During Implementation

- `fix_vod.py` - Python script used to fix malformed code
- `fix_main.ps1` - PowerShell script (not used in final solution)
- `update_vod.ps1` - PowerShell script (not used in final solution)
- `vod_all_movies.patch` - Patch file (reference only)

**Note:** These helper files can be deleted after verification.

---

### Next Steps

1. ✅ Test with actual IPTV playlist to verify "All Movies" functionality
2. Monitor scrolling performance with large datasets
3. Consider adding loading indicators for "All Movies" (can take longer)
4. Optional: Add a count indicator showing total movies in "All Movies" category
