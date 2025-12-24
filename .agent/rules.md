# 🔐 Matrix IPTV Security Rules

These rules are MANDATORY for all future updates and automated deployments.

### 1. Zero-Trust Credentials

- **NEVER** push files containing the following IPTV keywords with real data:
  - `pledge78502.cdn-akm.me`
  - `line.offcial-trex.pro`
  - Any URL ending in `.pro`, `.me`, `.xyz`, or `.vip` that looks like a stream provider.
  - Test usernames (e.g., `7c34d33c9e21`, `3a6aae52fb`).
- **Placeholder Rule**: Only use `YOUR_USERNAME`, `YOUR_PASSWORD`, and `http://your-provider.com` in code files.

### 2. File Lockdown (.gitignore)

- Keep all `.json` files ignored by default.
- If a new JSON file is needed for the app, it must be explicitly whitelisted in `.gitignore` with a `!` prefix ONLY IF it contains no private data.
- All `.log` and `.txt` files are forbidden from the repository.

### 3. Utility Script Hygiene

- Utility scripts in `src/bin/` (like `verify_login.rs` or `analyze_playlists.rs`) must be purged of all real account data before any commit.

### 4. Automated Guardrail

- The `github_deploy.ps1` script is the "Single Source of Truth" for pushing. It must include logic to check for uncommitted sensitive files.

### 5. Memory Instruction

- Antigravity MUST scan for these patterns before every `git push` or `git commit` action.
