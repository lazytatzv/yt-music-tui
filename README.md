# MELODY - Pro Music Player

The ultimate TUI music streaming experience with Vim keybindings and a beautiful, colorful interface. Built in Rust with yt-dlp for searching YouTube, SoundCloud, Bandcamp, Dailymotion, Vimeo, and Twitch.

## Features

- Multi-Platform Support - YouTube, SoundCloud, Bandcamp, Dailymotion, Vimeo, Twitch (easily extensible)
- Vim Keybindings - hjkl navigation, vim-like commands (g, G, etc.)
- Powerful Search - yt-dlp powered, supports all major video platforms
- Responsive TUI - Built with Ratatui & Crossterm
- Professional Design - RGB color palette, intuitive layout
- Platform-Specific Colors - Each platform has its own distinct color

## Quick Start

### Prerequisites

```bash
# Install yt-dlp
pip install yt-dlp

# Install FFmpeg (for audio playback)
sudo apt-get install ffmpeg

# Rust (https://rustup.rs/)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build & Run

```bash
cargo build --release
cargo run --release
```

Or run the binary directly:

```bash
./target/release/music-player
```

## Keybindings

### Navigation (Vim Style)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down in results |
| `k` / `↑` | Move up in results |
| `h` | Enter search mode |
| `l` | Play selected track |
| `g` | Go to first result |
| `G` | Go to last result |

### Search & Control

| Key | Action |
|-----|--------|
| `/` | Focus search bar |
| `Enter` | Execute search |
| `Esc` | Exit search mode |
| `Backspace` | Delete character |

### Playback

| Key | Action |
|-----|--------|
| `Space` | Play selected track |
| `p` | Pause/Resume |

### Other

| Key | Action |
|-----|--------|
| `Tab` | Cycle platform filter |
| `Ctrl+C` | Quit |

## Supported Platforms

- **YouTube** - Video hosting (Primary platform for music)
- **SoundCloud** - Independent artists & streaming
- **Bandcamp** - Independent music platform
- **Dailymotion** - Alternative video platform
- **Vimeo** - Creative video community
- **Twitch** - Live streaming & music sessions

Each platform has a unique color indicator for easy identification.

## Color Palette

- **Primary (Hot Pink)**: Main accent, selection
- **Secondary (Blue Violet)**: Header background
- **Accent (Turquoise)**: Borders, UI elements
- **Success (Mint)**: Platform filter active
- **Warning (Amber)**: Duration/status
- **Error (Coral)**: YouTube platform
- **Text (Light)**: Primary text

Platform-specific colors:
- Bandcamp: Purple
- Dailymotion: Orange
- Vimeo: Blue
- Twitch: Purple (Twitch brand)

## Project Structure

```
src/
├── main.rs               # Entry point
├── app.rs               # Application logic & event handling
├── player/mod.rs        # Player with yt-dlp integration
├── search/
│   ├── mod.rs           # SearchProvider trait
│   ├── youtube.rs       # YouTube search
│   ├── soundcloud.rs    # SoundCloud search
│   ├── bandcamp.rs      # Bandcamp search
│   ├── dailymotion.rs   # Dailymotion search
│   ├── vimeo.rs         # Vimeo search
│   └── twitch.rs        # Twitch search
└── ui/mod.rs            # TUI rendering & layout
```

## How to Extend

### Add a New Platform

1. Create `src/search/newplatform.rs`:

```rust
use async_trait::async_trait;
use anyhow::Result;
use crate::player::Track;
use crate::search::SearchProvider;

pub struct NewPlatformProvider;

#[async_trait]
impl SearchProvider for NewPlatformProvider {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<Track>> {
        // Implementation
        Ok(vec![])
    }

    fn platform_name(&self) -> &str {
        "NewPlatform"
    }
}
```

2. Add to `src/search/mod.rs`:
```rust
pub mod newplatform;
```

3. Add to `src/app.rs`:
```rust
Box::new(crate::search::newplatform::NewPlatformProvider),
```

4. Update platform cycling in `cycle_platform_filter()`:
```rust
let platforms = vec!["All", "YouTube", "SoundCloud", ..., "NewPlatform"];
```

### Customize Colors

Edit the `colors` module in `src/ui/mod.rs`:

```rust
pub mod colors {
    use ratatui::style::Color;
    pub const PRIMARY: Color = Color::Rgb(255, 107, 170);
    // ... customize colors
}
```

## Dependencies

- **tokio** - Async runtime
- **ratatui** - TUI framework
- **crossterm** - Terminal control
- **async-trait** - Async traits
- **anyhow** - Error handling
- **serde/serde_json** - JSON parsing
- **reqwest** - HTTP client

## Tips & Tricks

- **Quick search**: Press `/` to focus search and start typing
- **Browse platforms**: Use `Tab` to cycle between all supported platforms
- **Bulk navigation**: Use `g` (go start) and `G` (go end) for quick list navigation
- **Vi escape**: Press `Esc` to exit search mode and return to navigation

## Troubleshooting

### No audio output
```bash
# Ensure ffmpeg is installed
sudo apt-get install ffmpeg

# Check yt-dlp version
yt-dlp --version
```

### Search fails for certain platforms
```bash
# Update yt-dlp
pip install --upgrade yt-dlp
```

### Terminal colors not displaying correctly
- Use a terminal with 24-bit truecolor support
- Try: `export COLORTERM=truecolor`

## License

MIT

## Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) for incredible media downloading
- [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI framework
- [Tokio](https://tokio.rs/) for async runtime
