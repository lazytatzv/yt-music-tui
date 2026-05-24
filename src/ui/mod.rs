use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, BorderType},
    Frame,
};
use crate::player::Track;
use serde::{Serialize, Deserialize};
use chrono::Datelike;

// Modern Pastel Studio Palette
pub mod colors {
    use ratatui::style::Color;
    
    pub const BG: Color = Color::Rgb(22, 22, 30);
    pub const PANEL: Color = Color::Rgb(30, 32, 48);
    pub const PRIMARY: Color = Color::Rgb(255, 170, 190);  // Sakura Pink
    pub const SECONDARY: Color = Color::Rgb(150, 210, 255); // Sky Blue
    pub const ACCENT: Color = Color::Rgb(190, 170, 255);   // Lavender
    pub const SUCCESS: Color = Color::Rgb(170, 240, 190);  // Mint Green
    pub const WARNING: Color = Color::Rgb(255, 220, 170);  // Peach
    pub const TEXT: Color = Color::Rgb(160, 165, 185);
    pub const TEXT_BRIGHT: Color = Color::Rgb(240, 240, 245);
}

pub struct UI {
    pub search_query: String,
    pub selected_index: usize,
    pub search_results: Vec<Track>,
    pub current_track: Option<Track>,
    pub is_playing: bool,
    pub platform_filter: String,
    pub input_focus: bool,
    pub is_loading: bool,
    pub search_performed: bool,
    pub search_offset: usize,
    pub search_mode: SearchMode,
    pub search_history: Vec<String>,
    pub play_history: Vec<Track>,
    pub repeat_mode: RepeatMode,
    pub volume: u8,
    pub list_state: ratatui::widgets::ListState,
    pub playlists: Vec<Playlist>,
    pub current_playlist_idx: usize,
    pub playlist_list_state: ratatui::widgets::ListState,
    pub playlist_state: ratatui::widgets::ListState,
    pub history_state: ratatui::widgets::ListState,
    pub alarm_state: ratatui::widgets::ListState,
    pub view_mode: ViewMode,
    pub current_tab: Tab,
    pub focused_pane: Pane,
    pub is_renaming_playlist: bool,
    pub new_playlist_name: String,
    pub is_setting_alarm: bool,
    pub alarm_input: String,
    pub alarm_repeat_input: bool,
    pub alarm_loop_input: bool,
    pub alarm_track: Option<Track>,
    pub alarms: Vec<Alarm>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Pane {
    Main,
    Sidebar,
    Info,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Alarm {
    pub target_time: chrono::DateTime<chrono::Local>,
    pub track: Track,
    pub is_active: bool,
    pub repeat_daily: bool,
    pub infinite_loop: bool,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Home,
    Search,
    Library,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ViewMode {
    Home,
    Search,
    PlaylistList,
    PlaylistDetail,
}

#[derive(PartialEq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

#[derive(PartialEq, Clone, Copy)]
pub enum SearchMode {
    Track,
    Playlist,
}

impl UI {
    pub fn new() -> Self {
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(0));
        let mut playlist_list_state = ratatui::widgets::ListState::default();
        playlist_list_state.select(Some(0));
        let mut playlist_state = ratatui::widgets::ListState::default();
        playlist_state.select(Some(0));
        let mut history_state = ratatui::widgets::ListState::default();
        history_state.select(Some(0));
        let mut alarm_state = ratatui::widgets::ListState::default();
        alarm_state.select(Some(0));
        
        UI {
            search_query: String::new(),
            selected_index: 0,
            search_results: Vec::new(),
            current_track: None,
            is_playing: false,
            platform_filter: "All".to_string(),
            input_focus: false,
            is_loading: false,
            search_performed: false,
            search_offset: 0,
            search_mode: SearchMode::Track,
            search_history: Vec::new(),
            play_history: Vec::new(),
            repeat_mode: RepeatMode::Off,
            volume: 100,
            list_state,
            playlists: vec![Playlist { name: "My Favorites".to_string(), tracks: Vec::new() }],
            current_playlist_idx: 0,
            playlist_list_state,
            playlist_state,
            history_state,
            alarm_state,
            view_mode: ViewMode::Home,
            current_tab: Tab::Home,
            focused_pane: Pane::Main,
            is_renaming_playlist: false,
            new_playlist_name: String::new(),
            is_setting_alarm: false,
            alarm_input: String::new(),
            alarm_repeat_input: false,
            alarm_loop_input: false,
            alarm_track: None,
            alarms: Vec::new(),
        }
    }

    pub fn render(&mut self, f: &mut Frame, player: &crate::player::Player) {
        let area = f.area();
        
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(2), // Tabs
                Constraint::Min(0),    // Main Content
                Constraint::Length(6), // Advanced Player
            ])
            .split(area);

        self.draw_header(f, main_chunks[0]);
        self.draw_tabs(f, main_chunks[1]);
        
        // 3-Column Studio Layout
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Left Sidebar: Library
                Constraint::Percentage(60), // Center: Content
                Constraint::Percentage(20), // Right Panel: History/Info
            ])
            .split(main_chunks[2]);

        self.draw_sidebar(f, content_chunks[0]);
        self.draw_main_area(f, content_chunks[1], player);
        self.draw_info_panel(f, content_chunks[2], player);
        
        self.draw_advanced_player(f, main_chunks[3]);

        if self.is_renaming_playlist {
            self.draw_rename_modal(f);
        }
        if self.is_setting_alarm {
            self.draw_alarm_modal(f);
        }
    }

    fn draw_rename_modal(&mut self, f: &mut Frame) {
        let area = self.centered_rect(60, 20, f.area());
        let block = Block::default()
            .title(" RENAME PLAYLIST ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(colors::PRIMARY))
            .bg(colors::BG);
        
        let text = Line::from(vec![
            Span::styled(" Name: ", Style::default().fg(colors::SECONDARY)),
            Span::styled(&self.new_playlist_name, Style::default().fg(colors::TEXT_BRIGHT)),
            Span::styled("▊", Style::default().fg(colors::PRIMARY)),
        ]);
        
        let p = Paragraph::new(vec![
            Line::from(""),
            text,
            Line::from(""),
            Line::from(Span::styled(" [Enter] Save  │  [Esc] Cancel ", Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM))),
        ]).block(block).alignment(Alignment::Center);
        
        f.render_widget(ratatui::widgets::Clear, area); // This clears the background
        f.render_widget(p, area);
    }

    fn draw_alarm_modal(&mut self, f: &mut Frame) {
        let area = self.centered_rect(60, 30, f.area());
        let block = Block::default()
            .title(" SET AUDIO ALARM ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(colors::SUCCESS))
            .bg(colors::BG);
        
        let track_name = self.alarm_track.as_ref().map(|t| t.title.as_str()).unwrap_or("None");
        
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" Track: ", Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)),
                Span::styled(track_name, Style::default().fg(colors::TEXT_BRIGHT)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Enter Time: ", Style::default().fg(colors::SECONDARY)),
                Span::styled(&self.alarm_input, Style::default().fg(colors::TEXT_BRIGHT).add_modifier(Modifier::BOLD)),
                Span::styled("▊", Style::default().fg(colors::SUCCESS)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Repeat Daily: ", Style::default().fg(colors::SECONDARY)),
                Span::styled(if self.alarm_repeat_input { "[X] ON " } else { "[ ] OFF" }, 
                    if self.alarm_repeat_input { Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD) } 
                    else { Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM) }),
                Span::raw("   "),
                Span::styled(" Loop Audio: ", Style::default().fg(colors::SECONDARY)),
                Span::styled(if self.alarm_loop_input { "[X] ON " } else { "[ ] OFF" }, 
                    if self.alarm_loop_input { Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD) } 
                    else { Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM) }),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Format: ", Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)),
                Span::styled("'10'" , Style::default().fg(colors::ACCENT)),
                Span::styled(" (min)  ", Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)),
                Span::styled("'HH:MM'" , Style::default().fg(colors::ACCENT)),
                Span::styled(" (time)  ", Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)),
            ]),
            Line::from(vec![
                Span::styled(" [Tab/S-Tab] Toggle Settings  │  [Enter] Set Alarm  │  [Esc] Cancel ", Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)),
            ]),
        ]).block(block).alignment(Alignment::Center);
        
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(p, area);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
    fn draw_header(&mut self, f: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled(" MELODY STUDIO ", Style::default()
                .fg(colors::BG)
                .bg(colors::PRIMARY)
                .add_modifier(Modifier::BOLD)),
            Span::styled(" ❯ ", Style::default().fg(colors::PRIMARY)),
            Span::styled("HIGH-FIDELITY AUDIO WORKSTATION", Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)),
        ]);

        let header = Paragraph::new(title)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(colors::PANEL)));
        
        f.render_widget(header, area);
    }

    fn draw_tabs(&mut self, f: &mut Frame, area: Rect) {
        let tabs = vec![Tab::Home, Tab::Search, Tab::Library];
        let mut spans = Vec::new();

        for (i, tab) in tabs.iter().enumerate() {
            let label = match tab {
                Tab::Home => " HOME ",
                Tab::Search => " SEARCH ",
                Tab::Library => " LIBRARY ",
            };

            if self.current_tab == *tab {
                spans.push(Span::styled(label, Style::default().fg(colors::BG).bg(colors::PRIMARY).add_modifier(Modifier::BOLD)));
            } else {
                spans.push(Span::styled(label, Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)));
            }

            if i < tabs.len() - 1 {
                spans.push(Span::styled(" │ ", Style::default().fg(colors::PANEL)));
            }
        }

        f.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Center), area);
    }

    fn draw_sidebar(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.playlists.iter().enumerate().map(|(i, p)| {
            let is_current = i == self.current_playlist_idx;
            
            let prefix = if is_current {
                Span::styled(" ❯ ", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("   ", Style::default())
            };

            let name_style = if is_current {
                Style::default().fg(colors::TEXT_BRIGHT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::TEXT)
            };

            ListItem::new(Line::from(vec![
                prefix,
                Span::styled(&p.name, name_style),
            ]))
        }).collect();

        let list = List::new(items)
            .block(Block::default()
                .title(" ❯ COLLECTIONS ")
                .title_style(Style::default().fg(colors::SECONDARY).add_modifier(Modifier::BOLD))
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(colors::PANEL)));
        
        f.render_widget(list, area);
    }

    fn draw_main_area(&mut self, f: &mut Frame, area: Rect, player: &crate::player::Player) {
        if self.is_loading {
            self.draw_loading(f, area);
            return;
        }

        match self.current_tab {
            Tab::Home => self.draw_welcome(f, area),
            Tab::Search => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(area);
                self.draw_search_bar(f, chunks[0]);
                self.draw_results(f, chunks[1], player);
            }
            Tab::Library => {
                match self.view_mode {
                    ViewMode::PlaylistDetail => self.draw_playlist_detail(f, area, player),
                    _ => self.draw_playlist_list(f, area),
                }
            }
        }
    }

    fn draw_info_panel(&mut self, f: &mut Frame, area: Rect, _player: &crate::player::Player) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // History
                Constraint::Percentage(50), // Alarms
            ])
            .split(area);

        // History
        let history_idx = self.history_state.selected().unwrap_or(0);
        let history_items: Vec<ListItem> = self.search_history.iter().enumerate().map(|(i, h)| {
            let is_selected = i == history_idx && self.focused_pane == Pane::Info;
            
            let prefix = if is_selected {
                Span::styled(" ❯ ", Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD))
            } else {
                Span::styled(" ⌕ ", Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM))
            };

            let text_style = if is_selected {
                Style::default().fg(colors::TEXT_BRIGHT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::TEXT)
            };
            
            ListItem::new(Line::from(vec![
                prefix,
                Span::styled(h, text_style),
            ]))
        }).collect();

        let history_list = List::new(history_items)
            .block(Block::default()
                .title(" ❯ HISTORY ")
                .title_style(Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD))
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(if self.focused_pane == Pane::Info { colors::PANEL } else { colors::PANEL })));
        
        f.render_stateful_widget(history_list, chunks[0], &mut self.history_state);

        // Alarms
        let alarm_idx = self.alarm_state.selected().unwrap_or(0);
        let now = chrono::Local::now();
        let alarm_items: Vec<ListItem> = self.alarms.iter().enumerate().map(|(i, a)| {
            let is_selected = i == alarm_idx && self.focused_pane == Pane::Info;
            let diff = a.target_time.signed_duration_since(now).num_seconds();
            let (mins, secs) = if diff > 0 {
                (diff / 60, diff % 60)
            } else {
                (0, 0)
            };
            
            let prefix = if is_selected {
                Span::styled(" ❯ ", Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD))
            } else {
                Span::styled(" ● ", Style::default().fg(colors::WARNING))
            };

            let date_str = if a.target_time.date_naive() != now.date_naive() {
                format!("{}/{} ", a.target_time.month(), a.target_time.day())
            } else {
                "".to_string()
            };

            let style = if is_selected {
                Style::default().fg(colors::TEXT_BRIGHT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::TEXT)
            };

            let repeat_tag = if a.repeat_daily {
                Span::styled(" [R]", Style::default().fg(colors::ACCENT).add_modifier(Modifier::DIM))
            } else {
                Span::raw("")
            };

            let loop_tag = if a.infinite_loop {
                Span::styled(" [L]", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::DIM))
            } else {
                Span::raw("")
            };

            ListItem::new(Line::from(vec![
                prefix,
                Span::styled(date_str, style.add_modifier(Modifier::DIM)),
                Span::styled(format!("{:02}:{:02} ", mins, secs), style),
                Span::styled(&a.track.title, if is_selected { style } else { Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM) }),
                repeat_tag,
                loop_tag,
            ]))
        }).collect();

        let alarm_list = List::new(alarm_items)
            .block(Block::default()
                .title(" ❯ ALARMS ")
                .title_style(Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD))
                .borders(Borders::LEFT | Borders::TOP)
                .border_style(Style::default().fg(if self.focused_pane == Pane::Info { colors::SUCCESS } else { colors::PANEL })));
        
        f.render_stateful_widget(alarm_list, chunks[1], &mut self.alarm_state);
    }

    fn draw_search_bar(&mut self, f: &mut Frame, area: Rect) {
        let mode_label = match self.search_mode {
            SearchMode::Track => " SONG ",
            SearchMode::Playlist => " MIX ",
        };

        let content = Line::from(vec![
            Span::styled(mode_label, Style::default().fg(colors::BG).bg(colors::SECONDARY).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(&self.search_query, Style::default().fg(colors::TEXT_BRIGHT)),
            if self.input_focus { Span::styled("▊", Style::default().fg(colors::PRIMARY)) } else { Span::raw("") },
        ]);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if self.input_focus { colors::PRIMARY } else { colors::PANEL }));

        f.render_widget(Paragraph::new(content).block(block), area);
    }

    fn draw_results(&mut self, f: &mut Frame, area: Rect, player: &crate::player::Player) {
        let mut items = Vec::new();
        for (idx, track) in self.search_results.iter().enumerate() {
            let is_selected = idx == self.selected_index;
            let is_cached = player.is_cached(track);
            let is_saved = self.is_in_library(track);

            let prefix = if is_selected {
                Span::styled(" ❯ ", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("   ", Style::default())
            };

            let text_style = if is_selected {
                Style::default().fg(colors::TEXT_BRIGHT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::TEXT)
            };

            let platform_tag = match track.platform.as_str() {
                "YouTube" => " YT ",
                "SoundCloud" => " SC ",
                _ => " ?? ",
            };

            let duration_str = track.duration.map_or("--:--".to_string(), |d| {
                format!("{:02}:{:02}", d / 60, d % 60)
            });

            let title = if track.title.chars().count() > area.width as usize - 30 {
                let truncated: String = track.title.chars().take(27).collect::<String>();
                format!("{}...", truncated)
            } else {
                track.title.clone()
            };

            let tags = vec![
                if is_cached { Some(Span::styled(" CACHE ", Style::default().fg(colors::BG).bg(colors::SUCCESS))) } else { None },
                if is_saved { Some(Span::styled(" SAVED ", Style::default().fg(colors::BG).bg(colors::ACCENT))) } else { None },
            ];

            let mut row = Vec::new();
            row.push(prefix);
            row.push(Span::styled(format!("{:<4} │ ", platform_tag), Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)));
            row.push(Span::styled(title, text_style));
            row.push(Span::styled(format!(" │ {}", duration_str), if is_selected { text_style } else { Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM) }));
            
            for tag in tags.into_iter().flatten() {
                row.push(Span::raw(" "));
                row.push(tag);
            }

            items.push(ListItem::new(vec![
                Line::from(row),
                Line::from(Span::styled("─".repeat(area.width as usize), Style::default().fg(colors::PANEL).add_modifier(Modifier::DIM))),
            ]));
        }

        let list = List::new(items)
            .block(Block::default()
                .title(format!(" ❯ {} RESULTS ", self.search_results.len()))
                .title_style(Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)));

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_advanced_player(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(colors::PANEL).add_modifier(Modifier::BOLD));
        
        let inner = block.inner(area);
        f.render_widget(block, area);

        if let Some(track) = &self.current_track {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40), // Now Playing
                    Constraint::Percentage(40), // Progress
                    Constraint::Percentage(20), // Master Volume
                ])
                .split(inner);

            // 1. Now Playing
            let status = if self.is_playing { "PLAYING" } else { "PAUSED" };
            let color = if self.is_playing { colors::SUCCESS } else { colors::WARNING };
            let info = vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", status), Style::default().fg(colors::BG).bg(color).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(&track.title, Style::default().fg(colors::TEXT_BRIGHT).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(format!("  SOURCE: {}", track.platform), Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)),
                ]),
            ];
            f.render_widget(Paragraph::new(info), chunks[0]);

            // 2. Progress Simulation
            let progress_bar = "⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯";
            f.render_widget(Paragraph::new(vec![
                Line::from("PROGRESS ANALYSIS"),
                Line::from(Span::styled(progress_bar, Style::default().fg(colors::PANEL))),
            ]).alignment(Alignment::Center), chunks[1]);

            // 3. Master Volume
            let vol_filled = (self.volume / 10) as usize;
            let vol_bar = format!("MSTR [ {}{} ]", "█".repeat(vol_filled), " ".repeat(10 - vol_filled));
            f.render_widget(Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(vol_bar, Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD))),
            ]).alignment(Alignment::Right), chunks[2]);
        } else {
            f.render_widget(Paragraph::new("⎯ SYSTEM STANDBY ⎯").alignment(Alignment::Center).style(Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM)), inner);
        }
    }

    // Helper stubs to maintain compatibility with existing code
    fn draw_welcome(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let title = Paragraph::new(Line::from(vec![
            Span::styled(" ❯ MELODY MANUAL ", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)),
        ])).alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        let help_text = vec![
            Line::from(vec![Span::styled(" ❯ NAVIGATION ", Style::default().fg(colors::SECONDARY).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("   h / l          ", Style::default().fg(colors::SECONDARY)), Span::raw(" Switch between tabs (Home, Search, Library)")]),
            Line::from(vec![Span::styled("   Tab            ", Style::default().fg(colors::SECONDARY)), Span::raw(" Cycle through panels (Sidebar, Main, Info)")]),
            Line::from(vec![Span::styled("   j / k          ", Style::default().fg(colors::SECONDARY)), Span::raw(" Move selection up / down")]),
            Line::from(vec![Span::styled("   Enter / Space  ", Style::default().fg(colors::SECONDARY)), Span::raw(" Select playlist / Play selected track")]),
            Line::from(vec![Span::styled("   Esc            ", Style::default().fg(colors::SECONDARY)), Span::raw(" Go back / Exit search mode")]),
            Line::from(""),
            Line::from(vec![Span::styled(" ❯ SEARCH ", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("   /              ", Style::default().fg(colors::PRIMARY)), Span::raw(" Focus search bar (and switch to Search tab)")]),
            Line::from(vec![Span::styled("   Tab            ", Style::default().fg(colors::PRIMARY)), Span::raw(" Toggle between Track and Playlist search")]),
            Line::from(vec![Span::styled("   Enter          ", Style::default().fg(colors::PRIMARY)), Span::raw(" Execute search query")]),
            Line::from(""),
            Line::from(vec![Span::styled(" ❯ LIBRARY ", Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("   a              ", Style::default().fg(colors::ACCENT)), Span::raw(" Add selected search result to current playlist")]),
            Line::from(vec![Span::styled("   n              ", Style::default().fg(colors::ACCENT)), Span::raw(" Create a new playlist")]),
            Line::from(vec![Span::styled("   r              ", Style::default().fg(colors::ACCENT)), Span::raw(" Rename selected playlist (in Library list)")]),
            Line::from(vec![Span::styled("   d              ", Style::default().fg(colors::ACCENT)), Span::raw(" Delete selected playlist or track")]),
            Line::from(""),
            Line::from(vec![Span::styled(" ❯ PLAYER ", Style::default().fg(colors::SUCCESS).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("   p              ", Style::default().fg(colors::SUCCESS)), Span::raw(" Pause / Resume music")]),
            Line::from(vec![Span::styled("   s              ", Style::default().fg(colors::SUCCESS)), Span::raw(" Stop playback (clear player)")]),
            Line::from(vec![Span::styled("   r              ", Style::default().fg(colors::SUCCESS)), Span::raw(" Toggle repeat mode (Off / One / All)")]),
            Line::from(vec![Span::styled("   c / C          ", Style::default().fg(colors::SUCCESS)), Span::raw(" Download to cache / Delete from cache")]),
            Line::from(""),
            Line::from(vec![Span::styled(" ❯ ALARM SYSTEM ", Style::default().fg(colors::WARNING).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("   A              ", Style::default().fg(colors::WARNING)), Span::raw(" Set 3-minute alarm for selected track")]),
            Line::from(vec![Span::styled("   Tab            ", Style::default().fg(colors::WARNING)), Span::raw(" Switch focus to Info panel to manage alarms")]),
            Line::from(vec![Span::styled("   d              ", Style::default().fg(colors::WARNING)), Span::raw(" Cancel selected alarm (when Info focused)")]),
            Line::from(""),
            Line::from(vec![Span::styled(" ❯ VOLUME ", Style::default().fg(colors::WARNING).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("   1 - 0          ", Style::default().fg(colors::WARNING)), Span::raw(" Instant volume jump (10% to 100%)")]),
            Line::from(vec![Span::styled("   [ / ]          ", Style::default().fg(colors::WARNING)), Span::raw(" Fine-tune volume (+/- 5%)")]),
            Line::from(vec![Span::styled("   m              ", Style::default().fg(colors::WARNING)), Span::raw(" Mute / Unmute")]),
        ];

        let p = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors::PANEL)));

        f.render_widget(p, chunks[1]);
    }

    fn draw_loading(&mut self, f: &mut Frame, area: Rect) {
        f.render_widget(Paragraph::new("⎯ FETCHING DATASTREAM ⎯").alignment(Alignment::Center).fg(colors::PRIMARY), area);
    }

    fn draw_playlist_list(&mut self, f: &mut Frame, area: Rect) {
        let mut items = Vec::new();
        let selected_idx = self.playlist_list_state.selected().unwrap_or(0);

        for (idx, playlist) in self.playlists.iter().enumerate() {
            let is_selected = idx == selected_idx;
            
            let prefix = if is_selected {
                Span::styled(" ❯ ", Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("   ", Style::default())
            };

            let text_style = if is_selected {
                Style::default().fg(colors::TEXT_BRIGHT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::TEXT)
            };

            items.push(ListItem::new(Line::from(vec![
                prefix,
                Span::styled(&playlist.name, text_style),
                Span::styled(format!(" │ {} TUNES", playlist.tracks.len()), if is_selected { text_style } else { Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM) }),
            ])));
        }

        let list = List::new(items)
            .block(Block::default()
                .title(" ❯ DIRECTORIES ")
                .title_style(Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD))
                .borders(Borders::NONE));

        f.render_stateful_widget(list, area, &mut self.playlist_list_state);
    }

    fn draw_playlist_detail(&mut self, f: &mut Frame, area: Rect, player: &crate::player::Player) {
        let playlist = &self.playlists[self.current_playlist_idx];
        if playlist.tracks.is_empty() {
            let empty = Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![Span::styled("  NO TRACKS REGISTERED", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::ITALIC))]),
            ]);
            f.render_widget(empty, area);
            return;
        }

        let mut items = Vec::new();
        let selected_idx = self.playlist_state.selected().unwrap_or(0);

        for (idx, track) in playlist.tracks.iter().enumerate() {
            let is_selected = idx == selected_idx;
            let is_cached = player.is_cached(track);
            
            let prefix = if is_selected {
                Span::styled(" ❯ ", Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("   ", Style::default())
            };

            let text_style = if is_selected {
                Style::default().fg(colors::TEXT_BRIGHT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::TEXT)
            };

            let duration_str = track.duration.map_or("--:--".to_string(), |d| {
                format!("{:02}:{:02}", d / 60, d % 60)
            });

            let title = if track.title.chars().count() > area.width as usize - 30 {
                let truncated: String = track.title.chars().take(area.width as usize - 33).collect();
                format!("{}...", truncated)
            } else {
                track.title.clone()
            };

            let mut row = Vec::new();
            row.push(prefix);
            row.push(Span::styled(title, text_style));
            row.push(Span::styled(format!(" │ {}", duration_str), if is_selected { text_style } else { Style::default().fg(colors::TEXT).add_modifier(Modifier::DIM) }));
            
            if is_cached {
                row.push(Span::raw(" "));
                row.push(Span::styled(" CACHE ", if is_selected { Style::default().fg(colors::BG).bg(colors::PRIMARY) } else { Style::default().fg(colors::BG).bg(colors::SUCCESS) }));
            }

            items.push(ListItem::new(vec![
                Line::from(row),
                Line::from(Span::styled("─".repeat(area.width as usize), Style::default().fg(colors::PANEL).add_modifier(Modifier::DIM))),
            ]));
        }

        let list = List::new(items)
            .block(Block::default()
                .title(Span::styled(format!(" ❯ CONTENTS: {} ", playlist.name.to_uppercase()), Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD)))
                .borders(Borders::NONE));

        f.render_stateful_widget(list, area, &mut self.playlist_state);
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.search_results.len().saturating_sub(1) {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn is_in_library(&self, track: &Track) -> bool {
        self.playlists.iter().any(|p| p.tracks.contains(track))
    }

    pub fn get_selected_track(&self) -> Option<Track> {
        match self.view_mode {
            ViewMode::Search => self.search_results.get(self.selected_index).cloned(),
            ViewMode::PlaylistDetail => {
                let playlist = &self.playlists[self.current_playlist_idx];
                self.playlist_state.selected().and_then(|i| playlist.tracks.get(i).cloned())
            }
            _ => None,
        }
    }

    pub fn save_library(&self) -> anyhow::Result<()> {
        let data = serde_json::json!({
            "playlists": self.playlists,
            "history": self.search_history,
            "alarms": self.alarms,
        });
        let f = std::fs::File::create("library.json")?;
        serde_json::to_writer_pretty(f, &data)?;
        Ok(())
    }

    pub fn load_library(&mut self) {
        if let Ok(f) = std::fs::File::open("library.json") {
            let data: serde_json::Value = serde_json::from_reader(f).unwrap_or_default();
            if let Some(playlists) = data.get("playlists").and_then(|v| serde_json::from_value::<Vec<Playlist>>(v.clone()).ok()) {
                if !playlists.is_empty() {
                    self.playlists = playlists;
                }
            }
            if let Some(history) = data.get("history").and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok()) {
                self.search_history = history;
            }
            if let Some(alarms) = data.get("alarms").and_then(|v| serde_json::from_value::<Vec<Alarm>>(v.clone()).ok()) {
                // Only load alarms that are still in the future
                let now = chrono::Local::now();
                self.alarms = alarms.into_iter().filter(|a| a.target_time > now).collect();
            }
        }
    }
}
