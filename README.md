# 🟢 Matrix IPTV CLI

![Matrix IPTV CLI Hero](./assets/hero.png)

**// THE_PREMIUM_TERMINAL_IPTV_DECODER //**

Matrix IPTV CLI is a blazing fast, keyboard-driven interface for browsing Live TV, Movies, and Series. Inspired by the Matrix and built with Rust, it's designed to be the ultimate premium TUI (Terminal User Interface) for IPTV power users.

This project is being actively optimized by **Ernesto "Beats"** with a primary focus on **Live TV and Sports** performance, ensuring zero-latency navigation and professional-grade video clarity.

---

## 📸 Gallery

<p align="center">
  <img src="./assets/playlists.png" width="45%" />
  <img src="./assets/pill.png" width="45%" />
  <br />
  <img src="./assets/live.png" width="90%" />
</p>

---

## 🚀 Instant Installation

**Prerequisites:** Matrix IPTV CLI requires **MPV Player** (for video) and **Node.js** (for the CLI).

#### **Don't have MPV Player?**

- **Windows:** `winget install mpv`
- **Mac:** `brew install mpv`
- **Linux:** `sudo apt install mpv`

#### **Don't have Node.js?**

- **Windows:** `winget install OpenJS.NodeJS`
- **Mac:** `brew install node`
- **Linux:** `sudo apt install nodejs npm`

### **Global NPM Install (Recommended)**

If you have Node.js installed, this is the easiest way to stay updated:

```bash
npm install -g @officebeats/matrix-iptv-cli
```

### **One-Click Scripts**

Alternatively, use these platform-specific one-liners:

#### **Windows**

```powershell
powershell -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/officebeats/matrix-iptv/main/install.ps1 | iex"
```

#### **Mac & Linux**

```bash
curl -sSL https://raw.githubusercontent.com/officebeats/matrix-iptv/main/install.sh -o install_matrix.sh && bash install_matrix.sh && rm install_matrix.sh
```

---

## 🎬 How to Run

The app will **auto-launch** instantly after installation! 🚀

For future sessions, simply open any terminal and type:

```bash
matrix-iptv
```

---

## ✨ Features

- **🛡️ Playlist Modes**:
  - **'Merica Mode**: Filters for US/UK/Canada content, removes international clutter from **Strong 8K**, **Mega OTT**, & **TRex**. Renames "Football" to "Soccer".
  - **Sports Mode**: Prioritizes live sports events and adds league icons (NBA, NFL, MLB, NHL) for rapid identification.
- **⚡ Instant Response**: Built in Rust for maximum performance. Navigate tens of thousands of channels with zero lag.
- **📡 Xtream API Native**: Strictly optimized for **Xtream API** providers (support for M3U is not planned at this time).
- **🔎 Global Search**: Press **`Alt`+`Space`** (or `/`) to search across Live TV, Movies, and Series content instantly.
- **📁 Unified Navigation**: "All" views for Channels, Movies, and Series allow for rapid browsing without digging into categories.
- **🎬 Full Series Support**: dedicated browsing for TV Shows with Season/Episode hierarchy and tracking.
- **🎞️ VOD Experience**: Browse movies with rich metadata and instant playback.
- **📟 Matrix Rain Screensaver**: A high-performance, authentic digital rain screensaver that activates when idle.
- **🔒 Secure**: Uses Private DNS-over-HTTPS (DoH) for secure provider connection.
- **👁️ Headless Playback**: Integrated borderless fullscreen mode for a cinematic experience.

---

## 🛡️ Playlist Modes

Matrix IPTV CLI features advanced preprocessing engines called **Playlist Modes**. These filters run _before_ content reaches your screen to optimize the experience.

#### **'Merica Mode 🇺🇸**

Strictly optimized for **Strong 8K**, **Mega OTT**, and **TRex IPTV** playlists.

- **Geo-Filtering**: Hides international categories (AR, FR, DE, etc.) to focus on English-speaking content.
- **Name Cleaning**: Intelligent renaming (e.g., removes `US |`, `USA |`, `FHD`) for cleaner lists.
- **Sports Renaming**: Renames "American Football" to "Football" and "Football" to "Soccer".

#### **Sports Mode 🏟️**

Designed for the game day power user.

- **League Icons**: Automatically detects and prefixes league names with icons (🏀 NBA, 🏈 NFL, ⚾ MLB, 🏒 NHL).
- **Category Sorting**: Hoists sports categories to the top of the list for quick access.

To change modes:

1. Press **`m`** (or Go to Settings > Playlist Mode).
2. Select your desired mode.
3. Press **`Enter`**. The app will reload your playlist with the new optimizations.

---

## 📽️ MPV Enhancements

We leverage advanced **MPV** flags to ensure professional-level video quality even on lower-end hardware:

- **Headless Fullscreen**: Launches directly into borderless fullscreen (`--no-border`, `--fs`) for a professional, TV-like experience.
- **Advanced Anti-Aliasing**: Uses `spline36` scaling, providing superior edge smoothing.
- **Oversample Upscaling**: High-quality temporal upscaling that sharpens images.
- **Motion Smoothing**: High-performance `display-resample` interpolation for fluid sports playback.
- **Hardware Acceleration**: Automatic `hwdec=auto-safe` with modern Windows `d3d11-flip` presentation.

---

## ⌨️ Common Controls

| Key                     | Action                                |
| :---------------------- | :------------------------------------ |
| **`Enter`**             | **Play Channel / Select / Confirm**   |
| **`Esc` / `Backspace`** | **Go Back / Cancel**                  |
| **`/`**                 | **Search** (Filters current view)     |
| **`Alt` + `Space`**     | **Global Search** (Search everything) |
| **`j` / `↓`**           | Move Down                             |
| **`k` / `↑`**           | Move Up                               |
| **`m`**                 | **Playlist Mode** (Quick Switch)      |
| **`x`**                 | **Settings**                          |
| **`r`**                 | **Refresh Playlist**                  |
| **`q`**                 | **Quit**                              |

---

## 🛠️ Prerequisites

The installation scripts will attempt to install these for you:

- **MPV Player**: [mpv.io](https://mpv.io)
- **Rust Compiler**: [rustup.rs](https://rustup.rs) (Only required for manual builds)

---

> **⚠️ Disclaimer:** Matrix IPTV CLI and its creator, are **not affiliated** with z2u, g2g, or any IPTV provider. We do not sell or distribute content. All transactions on these platforms are at your own risk. This guide is for informational purposes only (experimental "USA Mode" testing).

<details>
<summary><strong>🛒 Where to Buy Playlists (Click to Expand)</strong></summary>

<br>

The experimental "USA Mode" is optimized for **Strong 8K**, **TRex**, and **Mega OTT** playlists. These are typically sourced from third-party marketplaces:

- **Platforms**: **z2u.com** or **g2g.com**
- **Search Terms**: "Strong 8k IPTV", "Trex IPTV", "Mega OTT"
- **Duration**: Usually sold in **1-month**, **6-month**, or **1-year** increments.

### ✅ Buying Tips

1.  **Check Ratings**: Always choose a seller with a **high rating (98%+)** and a high sales count. These are 2-sided marketplaces, so reputation is everything.
2.  **Safe Payment**: Use strictly secure payment methods like **Google Pay** or **Apple Pay** directly through your device. Avoid direct bank transfers or obscure payment links.

</details>

---

## ⚡ Community & Support

Built and optimized with ❤️ by **Ernesto "Beats"** with the help of google antigravity and vibe coding during his PTO vacation time.

[![Twitter](https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white)](https://x.com/officebeats)
[![Discord](https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/users/317887730703138826)

---

## 📜 License

MIT // [ProductMG.com](https://www.productmg.com)
