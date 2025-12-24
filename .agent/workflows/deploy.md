---
description: How to deploy Matrix IPTV to GitHub
---

Follow these steps to upload your code to GitHub and trigger automated builds.

### 1. Initialize Git (If not already done)

```bash
git init
git add .
git commit -m "Initial commit: Matrix IPTV System"
```

### 2. Create a GitHub Repository

Go to [GitHub](https://github.com/new) and create a new repository named `matrix-iptv`.

### 3. Connect Local Repo to GitHub

Replace `[your-username]` with your actual GitHub username:

```bash
git remote add origin https://github.com/[your-username]/matrix-iptv.git
git branch -M main
git push -u origin main
```

### 4. Create a Release (to trigger automated builds)

The provided GitHub Action triggers on tags starting with `v` (e.g., `v1.0.0`).

```bash
git tag v1.0.0
git push origin v1.0.0
```

This will:

1. Push the code to GitHub.
2. Trigger the Build Action in `.github/workflows/release.yml`.
3. Build binaries for **Windows**, **Linux**, and **macOS**.
4. Create a "Release" on your GitHub page with the downloadable files attached automatically.

### 5. Future Updates

Whenever you make changes:

```bash
git add .
git commit -m "Description of changes"
git push origin main
```

To trigger a new downloadable release, just create a new tag (e.g., `v1.0.1`).
