# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Matrix IPTV CLI is a Rust TUI application for browsing and playing IPTV streams (Live TV, Movies, Series) via Xtream API providers. Built with Ratatui + Crossterm, async Tokio runtime, distributed via NPM (`@officebeats/matrix-iptv-cli`) with pre-built binaries for Windows, Linux, and macOS.

## Build & Development Commands

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo run                    # Run the app
cargo test                   # Run all tests
cargo test <test_name>       # Run a single test
cargo fmt                    # Format code
cargo check                  # Type-check without building
cargo build --features chromecast  # Build with Chromecast support (requires OpenSSL on Windows)
cargo run --bin <binary_name>      # Run utility binaries from src/bin/
```

## Architecture

### State Management (`src/app.rs`)

The `App` struct is the single source of truth for all application state. Key patterns:

- **Screen navigation**: `CurrentScreen` enum (16 variants: `Home`, `Login`, `Categories`, `Streams`, `VodCategories`, `VodStreams`, `SeriesCategories`, `SeriesStreams`, `Settings`, `TimezoneSettings`, `ContentTypeSelection`, `GlobalSearch`, `GroupManagement`, `GroupPicker`, `UpdatePrompt`, `SportsDashboard`)
- **Pane focus**: `Pane` enum (`Categories`, `Streams`, `Episodes`) — three-column layout with focus tracking
- **Dual-list pattern**: Each content type maintains parallel lists — `all_categories`/`categories`, `all_streams`/`streams`, etc. The `all_*` lists hold unfiltered data; the display lists hold the search/filter-applied view. This pattern repeats for VOD and Series.
- **Selection state**: `selected_*_index` + `ListState` pairs per list for navigation and rendering

### Main Loop (`src/main.rs`)

The app uses a non-blocking event loop:

1. **Draw** — `terminal.draw(|f| ui::ui(f, app))`
2. **Process async actions** — drain `mpsc::channel::<AsyncAction>(32)` via `try_recv()`
3. **Debounced side effects** — EPG fetch (300ms), stream health check (1s), VOD info (500ms)
4. **Event handling** — keyboard/mouse input routed through `handlers::handle_key_event()`

CLI flags: `--play <URL>` (direct playback, no TUI), `--check` (verify config), `--skip-update`.

Exit code 42 signals the NPM wrapper (`bin/cli.js`) to trigger a binary update.

### Async Action System (`src/handlers/async_actions.rs`)

Background work uses a channel-based pattern. The `AsyncAction` enum has 55+ variants covering login, content loading, metadata fetching, playback, scores, casting, and system events. Handlers enqueue actions via `tx.send()`, the main loop processes them, and each action updates `App` state and may spawn follow-up tasks (e.g., `LoginSuccess` spawns parallel category loaders for live/VOD/series).

### Input Handling (`src/handlers/input.rs`)

Input is priority-ordered (checked top to bottom):
1. Global hotkeys (`Ctrl+Space` → search, etc.)
2. Loading state (only Esc allowed)
3. Overlay/popup interactions (help, guide, play details, cast picker, errors)
4. Screen-specific input (match on `CurrentScreen`)

Long operations spawn `tokio::spawn()` tasks that send `AsyncAction` via the channel.

### UI Rendering (`src/ui/`)

`ui::ui()` dispatches to screen-specific renderers based on `CurrentScreen`, then applies an overlay stack (highest priority last): loading spinner → matrix rain → guide popup → play details → cast picker → error overlay.

Main layout: Header (2-3 lines) | Content (horizontal: Categories pane | Streams pane) | Footer (1 line). Sports events split the right pane vertically to show match details.

Key submodules: `panes.rs` (multi-column lists), `popups.rs` (modals), `home.rs`, `series.rs`, `vod.rs`, `sports.rs` (screen renderers), `form.rs` (login/settings forms), `header.rs`/`footer.rs` (bars), `colors.rs` (palette), `common.rs` (shared widgets).

### Other Key Modules

- **`src/api.rs`** — Xtream API client (`IptvClient` enum wrapping `XtreamClient`), data models: `Category`, `Stream` (with cached parsed state, fuzzy match), `UserInfo`, `SeriesInfo`, `VodInfo`
- **`src/parser.rs`** — Stream name parsing, sports event detection, timezone handling. Heavy regex with pre-compiled `once_cell` patterns
- **`src/preprocessing.rs`** — Parallelized filtering (Rayon), playlist modes ('Merica, Sports, All-English), favorites
- **`src/player.rs`** — MPV/VLC integration, stream health checking, hardware acceleration
- **`src/config.rs`** — JSON config at platform-specific `ProjectDirs` path (`com.vibecoding.vibe-iptv`). Persists: accounts (multi-provider), favorites, playlist modes, DNS provider, player preferences, auto-refresh interval (default 12h), recently watched (max 20). Has migration logic from legacy `com.vibecoding.iptv-cli`
- **`src/scores.rs`** / **`src/sports.rs`** — Live sports scores (fetched on 60s interval)
- **`src/matrix_rain.rs`** — Screensaver effect
- **`src/cast.rs`** — Chromecast (behind `chromecast` feature flag)

### Crate Configuration

Library target is `matrix_iptv_lib` with crate types `cdylib` + `rlib`. WASM target (`src/wasm_client.rs`) uses reduced Tokio features. The `chromecast` feature is optional.

## Testing

Integration tests in `tests/`: `frontend_screens_test.rs` (Ratatui `TestBackend`), `caching_test.rs`, `playlist_verification.rs`. QA automation via `src/bin/qa_bot.rs`.

## Security Rules (MANDATORY)

- **Never commit real credentials.** Use placeholders: `YOUR_USERNAME`, `YOUR_PASSWORD`, `http://your-provider.com`
- **All `.json` files are gitignored by default.** Only `Cargo.toml`, `package.json`, and `test_config.json` are whitelisted
- **Purge real account data** from `src/bin/` utility scripts before any commit
- **Scan for provider URLs** (`.pro`, `.me`, `.xyz`, `.vip` domains) before pushing
- Use `github_deploy.ps1` as the deployment gateway — it includes config.json safety checks

## Deployment

CI/CD via `.github/workflows/release.yml`: manual `workflow_dispatch` with tag name input → builds release binaries on Windows/Linux/macOS → creates GitHub Release → publishes to NPM. The NPM package (`bin/cli.js`) handles binary selection per platform and auto-update checks.
