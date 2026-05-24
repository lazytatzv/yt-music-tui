# yt-tui 🎵

A sophisticated, fashionable, and high-fidelity YouTube Music TUI player for the terminal. Built with Rust, Ratatui, and mpv.

![License](https://img.shields.io/badge/license-MIT-pink.svg)
![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

## ✨ Features

- **Fashionable UI**: Aesthetically pleasing "Studio" layout with a soft pastel palette (Nord-inspired Sakura/Sky/Lavender).
- **Tabbed Navigation**: Intuitive `h` / `l` tab switching between Home, Search, and Library.
- **Smart Search**: Toggle between **Track** and **Playlist** search modes using `Tab`.
- **Advanced Library**: 
  - Multiple playlist support with renaming and deletion.
  - Persistent storage (`library.json`) for playlists and search history.
  - Duplicate prevention in playlists.
- **High-Performance Audio**:
  - Powered by `mpv` via IPC (Unix Socket) for gapless-like transition and instant response.
  - Real-time volume control with geometric visual feedback.
  - **Caching System**: Download tracks to local storage (`./cache`) with `c` key for instant offline playback.
- **Alarm System**: 
  - Set alarms for specific tracks using relative (`10` mins) or absolute (`14:30`) time formats.
  - Support for **Daily Repeat** and **Infinite Loop** (Snooze mode).
  - Alarms are persistent across restarts.
- **Studio Layout**: 3-column dashboard showing collections, search results, and recent history/alarms simultaneously.

## 🚀 Installation

### 1. Download Binary (Recommended)

Simply go to the [Releases](https://github.com/lazytatzv/yt-music-tui/releases) page and download the latest binary for your system.

### 2. Manual Installation

The installer will check for these, but you'll need:
- **Rust** (Cargo)
- **mpv**: Audio engine
- **yt-dlp**: YouTube metadata and streaming backend

### Quick Setup

```bash
git clone https://github.com/lazytatzv/yt-music-tui.git
cd yt-music-tui
chmod +x setup.sh
./setup.sh
```

## ⌨️ Keybindings

### Global & Navigation
| Key | Action |
|-----|--------|
| `h` / `l` | Switch Tabs (Home / Search / Library) |
| `Tab` | Cycle through Panels (Sidebar / Main / Info) |
| `j` / `k` | Move Selection Up / Down |
| `Enter` / `Space` | Select / Play |
| `Esc` | Back / Exit focus |
| `Ctrl+C` | Quit App |

### Search Tab
| Key | Action |
|-----|--------|
| `/` | Focus search bar |
| `Tab` | Toggle [song] / [mix] mode (while typing) |
| `a` | Add selected track to current playlist |
| `c` | Download/Cache track for instant playback |

### Library Tab
| Key | Action |
|-----|--------|
| `n` | Create new playlist |
| `r` | Rename selected playlist |
| `d` | Delete selected playlist or track |

### Player & Audio
| Key | Action |
|-----|--------|
| `p` | Pause / Resume |
| `s` | Stop (Clear player) |
| `r` | Toggle Repeat (Off / One / All) |
| `1` - `0` | Quick Volume Jump (10% - 100%) |
| `[` / `]` | Fine Volume Control (+/- 5%) |
| `m` | Mute / Unmute |
| `A` | Set Alarm for selected track |

## 📁 File Structure

- `cache/`: Local audio files (cached for speed)
- `library.json`: Persistent user data
- `melody.log`: Background logs (for debugging)

---
Made with ✦ for terminal music lovers.
