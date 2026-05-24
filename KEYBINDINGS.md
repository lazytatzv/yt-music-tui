# MELODY - Keybindings Reference

## Quick Reference

```
╭─ Navigation (Normal Mode) ──────────────────────────────────────────╮
│ j / ↓         Move down                                             │
│ k / ↑         Move up                                               │
│ g             Go to first result                                    │
│ G             Go to last result                                     │
╰─────────────────────────────────────────────────────────────────────╯

╭─ Playback ──────────────────────────────────────────────────────────╮
│ Space         Play selected track                                   │
│ p             Pause / Resume playback                               │
│ l             Play current selection (alternative to Space)         │
╰─────────────────────────────────────────────────────────────────────╯

╭─ Search Mode ───────────────────────────────────────────────────────╮
│ / or h        Enter search mode                                     │
│ ESC           Exit search mode                                      │
│ Enter         Execute search                                        │
│ Backspace     Delete character                                      │
│ [text]        Type search query                                     │
╰─────────────────────────────────────────────────────────────────────╯

╭─ Filters ──────────────────────────────────────────────────────────╮
│ Tab           Cycle platform (All → YouTube → SoundCloud → ...)    │
╰─────────────────────────────────────────────────────────────────────╯

╭─ Other ────────────────────────────────────────────────────────────╮
│ Ctrl+C        Quit application                                      │
╰─────────────────────────────────────────────────────────────────────╯
```

## Detailed Keybindings

### Navigation (Normal Mode - hjkl Vim Style)

| Key | Function | Notes |
|-----|----------|-------|
| `j` | Move selection down | Can also use ↓ arrow key |
| `k` | Move selection up | Can also use ↑ arrow key |
| `h` | Enter search mode | Focus on search input |
| `l` | Play selected track | Alternative to Space key |
| `g` | Go to first result | Vim-style "gg" alternative |
| `G` | Go to last result | Jump to end of list |

### Playback Control

| Key | Function | Toggle |
|-----|----------|--------|
| `Space` | Play selected track | - |
| `p` | Pause/Resume | Toggles between play and pause |

### Search Interface

| Key | Function | Context |
|-----|----------|---------|
| `/` | Enter search mode | Focus on search bar |
| `h` | Enter search mode | Alternative to `/` |
| `Enter` | Execute search | Submit query to all platforms |
| `Backspace` | Delete last character | While in search mode |
| `ESC` | Exit search mode | Return to navigation mode |
| `Tab` | Cycle platform filter | Works in both modes |

### Platform Filtering

| Key | Function | Cycles Through |
|-----|----------|-----------------|
| `Tab` | Next platform filter | All → YouTube → SoundCloud → Bandcamp → Dailymotion → Vimeo → Twitch → All |

### Application Control

| Key | Function | Notes |
|-----|----------|-------|
| `Ctrl+C` | Quit application | Graceful exit |

## Usage Tips

### Efficient Searching
```
1. Press "/" to enter search mode
2. Type your query: "anime opening"
3. Press Enter to search across all platforms
4. Use "j/k" to navigate results
5. Press Space to play
```

### Platform-Specific Search
```
1. Press Tab to filter to specific platform
2. Press "/" and enter search query
3. Results will only show from selected platform
```

### Quick Navigation
```
- Press "g" to jump to first result
- Press "G" to jump to last result
- Use "j" to scroll down through results
```

### Playing Music
```
1. Search for a track: "/" → "query" → Enter
2. Navigate to desired track: "jk"
3. Play with Space or "l"
4. Pause/Resume with "p"
```

## Mode Indicator

The search bar border color changes to indicate focus:

- 🔴 **Hot Pink** - Search mode active (focused)
- 🔵 **Turquoise** - Navigation mode (not focused)

## ASCII Diagram

```
MELODY Layout:

┌────────────────────────────────┐
│  Header with platform info     │  <- Shows current filter & tips
├────────────────────────────────┤
│  🔍 Search Bar (hjkl focus)    │  <- Color changes with focus
├────────────────────────────────┤
│  Results (navigate with jk)    │  <- Shows playable tracks
│  ▶ Track 1          00:01      │
│    Track 2          00:02      │
│    Track 3          00:03      │
├────────────────────────────────┤
│  🎧 Player Status              │  <- Shows current playing track
│     ▶ Track Title              │
│     [YouTube] ⏱ 01:28          │
└────────────────────────────────┘
```

## Advanced Workflows

### Batch Search Across Platforms
```
1. Start in "All" platform mode
2. Search query: "/" → "indie rock" → Enter
3. Results aggregate from YouTube, SoundCloud, Bandcamp, Dailymotion, Vimeo, Twitch
4. Select and play using Space
```

### Platform-Specific Browsing
```
1. Press Tab multiple times to reach desired platform (YouTube, Bandcamp, Vimeo, etc)
2. Search: "/" → "music query" → Enter
3. Only results from selected platform will appear
4. Tab again to switch to another platform
```

### Playing Playlist
```
1. Search: "/" → "playlist name" → Enter
2. Navigate down with "j"
3. Each track: Space to play → wait/pause → "j" to next → Space
```

## Accessibility Notes

- All navigation uses Vim keybindings (standard in many terminals)
- Mouse is disabled for pure keyboard workflow
- Clear visual feedback with color changes
- Emoji support for visual indicators

## Customization

See `src/ui/mod.rs` to customize:
- Color palette (RGB values)
- Keybindings in `src/app.rs`
- UI layout parameters

