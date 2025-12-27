# Matrix IPTV CLI - Troubleshooting Log

This document tracks identified issues and their resolutions to assist in future debugging and maintenance.

## Network & Connectivity

### [2025-12-26] Network Request Failed (DNS Settings)

- **Issue**: User encountered "Network request failed" error when trying to load a playlist.
- **Symptom**: Red error overlay showing `error sending request for url`.
- **Cause**: Custom DNS-over-HTTPS (DoH) providers (Quad9, Cloudflare, etc.) can sometimes fail depending on the local network environment or Windows SChannel compatibility.
- **Resolution**: Reverting the **DNS Provider** setting to **"System DNS"** in the Settings menu ('x') resolved the issue.
- **Future Check**: If network errors occur, prioritize validating if the DNS Provider is set correctly for the user's environment.

### [2025-12-26] Release Build Connection Errors (TLS)

- **Issue**: The application failed to connect to many providers in the release build.
- **Cause**: `rustls` (the default async TLS library) has strict requirements that many IPTV middleware servers do not meet.
- **Resolution**: Switched `reqwest` to use `native-tls` (SChannel on Windows) in `Cargo.toml`.

## Performance

### [2025-12-26] Slow Playlist Loading (>3 seconds)

- **Issue**: Selecting a playlist took several seconds to show the Content Type Selection screen.
- **Cause**:
  1. Blocking the UI thread with massive JSON parsing.
  2. Overloading the network/CPU with 6 simultaneous "Full Scan" requests.
  3. Inefficient filtering of foreign channels using $O(N \times M)$ string comparisons.
- **Resolution**:
  1. Moved JSON parsing to `spawn_blocking`.
  2. Staggered background scan requests with delays (500ms - 2.5s) to prioritize initial category loading.
  3. Combined 80+ foreign patterns into a single pre-compiled regex for $O(N)$ scanning.

## Resiliency

### [2025-12-26] "Silent Freeze" on Loading Screen

- **Issue**: Selection a playlist would sometimes hang on the "Secure Uplink" screen forever.
- **Cause**: Errors during async loading were not clearing the `loading_message` state.
- **Resolution**: Integrated a global error overlay and ensured that all `AsyncAction::Error` or `AsyncAction::LoginFailed` branches clear the loading state.

### [2025-12-26] Malformed API Responses

- **Issue**: Playlists with empty sections (Movies/Series) caused JSON parsing errors.
- **Cause**: Some providers return `{}` (empty object) or `null` instead of `[]` (empty list) for stream actions.
- **Resolution**: Added checks for empty object/null string bytes before attempting to deserialize into a `Vec`.

## UI & UX

### [2025-12-26] Persistent Loading Popup

- **Issue**: The loading popup remained visible after categories or streams were loaded.
- **Cause**: `loading_message` state was not cleared in the `AsyncAction` handlers, only `state_loading` was set to false.
- **Resolution**: Added `app.loading_message = None` to all data-loaded match arms in `src/handlers/async_actions.rs`.

### [2025-12-26] NBA/Sports American Mode Syntax

- **Issue**: Names in the "NBA Package" category were not being cleaned properly (prefixes remained).
- **Cause**: The `CLEAN_PREFIX_COMBINED` regex was missing common sports markers like NBA, NFL, etc.
- **Resolution**: Added `NBA|NFL|MLB|UFC|NHL|MLS` to the starting prefix cleaning regex in `src/parser.rs`.
