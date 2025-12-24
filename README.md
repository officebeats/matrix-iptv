# 🟢 Matrix IPTV CLI

**// THE_PREMIUM_TERMINAL_IPTV_DECODER //**

Matrix IPTV is a blazing fast, keyboard-driven interface for browsing Live TV, Movies, and Series. Inspired by the Matrix and built with Rust, it's designed to be the ultimate premium TUI (Terminal User Interface) for IPTV power users.

---

## 🚀 Instant Installation

Matrix IPTV is cross-platform. Copy and paste the command for your system below. The installer will automatically check for dependencies (MPV, Rust, Git) and set everything up for you.

### **Windows**

1. Open **Command Prompt** or **PowerShell**.
2. Paste this command and hit **Enter**:
   ```cmd
   powershell -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/officebeats/matrix-iptv/main/install.ps1 | iex"
   ```

### **Mac & Linux**

1. Open your **Terminal**.
2. Paste this command and hit **Enter**:
   ```bash
   curl -sSL https://raw.githubusercontent.com/officebeats/matrix-iptv/main/install.sh | bash
   ```

---

## 🎬 How to Run

The installer will automatically launch the app for you the first time. For future use, simply open any terminal and type:

```bash
matrix-iptv
```

---

## ✨ Features

- **⚡ Blazing Fast**: No more slow sluggish menus. Navigate thousands of channels instantly.
- **🎯 Provider Optimized**: Specifically tuned for **Trex** and **Strong8k** playlists for maximum speed and reliability.
- **📁 "All" Content Navigation**: Browse everything at once with "All Channels" and "All Movies" views.
- **🎬 Full Series Support**: Dedicated multi-column view for Series, Seasons, and Episodes.
- **📟 Infinite Screensaver**: High-performance Matrix rain screensaver (find it in Settings).
- **📡 Secure**: Uses Private DNS-over-HTTPS (DoH) for connecting to your provider.
- **📽️ Native Playback**: Uses the legendary **MPV** player for the smoothest possible video.

---

## 🏎️ Optimized Providers

Matrix IPTV is refined to work perfectly with:

- **Trex IPTV**: Optimized category parsing.
- **Strong8k**: Enhanced metadata and series support.

---

## ⌨️ Common Controls

Matrix IPTV is designed to be used without a mouse. It's faster that way.

| Key                     | Action                                                     |
| :---------------------- | :--------------------------------------------------------- |
| **`Enter`**             | **Play Channel / Select Category**                         |
| **`Esc` / `Backspace`** | **Go Back**                                                |
| **`/`**                 | **Search** (Filters results instantly)                     |
| **`j` / `↓`**           | Move Down                                                  |
| **`k` / `↑`**           | Move Up                                                    |
| **`l`**                 | Switch to **Live TV**                                      |
| **`v`**                 | Switch to **Movies** (VOD)                                 |
| **`s`**                 | Switch to **Series Mode**                                  |
| **`x`**                 | **Settings** (Update your playlist or turn on Screensaver) |
| **`n`**                 | **Add New Playlist** (New Uplink)                          |
| **`q`**                 | **Quit**                                                   |

---

## 🛠️ Prerequisites

The installation scripts will attempt to install these for you, but you can also get them manually:

- **MPV Player**: [mpv.io](https://mpv.io)
- **Rust Compiler**: [rustup.rs](https://rustup.rs)

---

## 🌐 Developers & Advanced

- **Build manually**: `cargo build --release`
- **Release Automation**: This repo includes GitHub Actions to automatically build Windows/Mac/Linux binaries.
- **GitHub**: [github.com/officebeats/matrix-iptv](https://github.com/officebeats/matrix-iptv)

---

## 📜 License

MIT // Created by Ernesto "Beats" // [ProductMG.com](https://www.productmg.com)
