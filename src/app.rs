use chrono::{Local, Timelike, NaiveTime, NaiveDateTime, Datelike};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;
use anyhow::Result;

use crate::player::Player;
use crate::search::SearchProvider;
use crate::ui::UI;

pub struct App {
    player: Player,
    ui: UI,
    search_providers: Vec<Box<dyn SearchProvider>>,
}

impl App {
    pub fn new() -> Self {
        let search_providers: Vec<Box<dyn SearchProvider>> = vec![
            Box::new(crate::search::youtube::YouTubeProvider),
        ];

        let mut ui = UI::new();
        ui.load_library();

        App {
            player: Player::new(),
            ui,
            search_providers,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.event_loop(&mut terminal).await;

        self.player.kill();

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res
    }

    async fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let mut last_tick = std::time::Instant::now();
        loop {
            terminal.draw(|f| {
                self.ui.render(f, &self.player);
            })?;

            // Tick logic (for alarms and auto-progression)
            if last_tick.elapsed() >= Duration::from_secs(1) {
                let mut triggered_track = None;
                let mut should_loop = false;
                let now = chrono::Local::now();
                for alarm in &mut self.ui.alarms {
                    if alarm.is_active && now >= alarm.target_time {
                        triggered_track = Some(alarm.track.clone());
                        should_loop = alarm.infinite_loop;
                        if alarm.repeat_daily {
                            alarm.target_time = alarm.target_time + chrono::Duration::days(1);
                        } else {
                            alarm.is_active = false;
                        }
                    }
                }
                
                let mut changed = false;
                if triggered_track.is_some() {
                    // Cleanup finished alarms
                    self.ui.alarms.retain(|a| a.is_active);
                    changed = true;
                }
                
                if let Some(track) = triggered_track {
                    let _ = self.player.play(track.clone()).await;
                    self.ui.current_track = Some(track);
                    self.ui.is_playing = true;
                    if should_loop {
                        self.player.set_repeat_mode("One");
                    }
                }

                // Auto-progression for ALL mode
                if self.ui.is_playing && self.player.is_idle() {
                    if self.ui.repeat_mode == crate::ui::RepeatMode::All {
                        let _ = self.play_next().await;
                    } else if self.ui.repeat_mode == crate::ui::RepeatMode::Off {
                        self.ui.is_playing = false;
                    }
                }

                if changed {
                    let _ = self.ui.save_library();
                }
                last_tick = std::time::Instant::now();
            }

            if self.ui.is_loading {
                self.perform_search().await?;
                continue;
            }

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if !self.handle_key_event(key).await? {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // Exit
            KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                return Ok(false);
            }
            KeyCode::Esc => {
                if self.ui.is_renaming_playlist {
                    self.ui.is_renaming_playlist = false;
                } else if self.ui.is_setting_alarm {
                    self.ui.is_setting_alarm = false;
                } else if self.ui.input_focus {
                    self.ui.input_focus = false;
                } else {
                    match self.ui.current_tab {
                        crate::ui::Tab::Library => {
                            if self.ui.view_mode == crate::ui::ViewMode::PlaylistDetail {
                                self.ui.view_mode = crate::ui::ViewMode::PlaylistList;
                            } else {
                                self.ui.current_tab = crate::ui::Tab::Home;
                            }
                        }
                        crate::ui::Tab::Search => {
                            self.ui.current_tab = crate::ui::Tab::Home;
                        }
                        _ => {}
                    }
                }
                return Ok(true);
            }

            // Text input / Rename / Alarm handling
            KeyCode::Char(c) if self.ui.input_focus || self.ui.is_renaming_playlist || self.ui.is_setting_alarm => {
                if self.ui.is_renaming_playlist {
                    self.ui.new_playlist_name.push(c);
                } else if self.ui.is_setting_alarm {
                    self.ui.alarm_input.push(c);
                } else {
                    self.ui.search_query.push(c);
                }
            }
            KeyCode::Backspace if self.ui.input_focus || self.ui.is_renaming_playlist || self.ui.is_setting_alarm => {
                if self.ui.is_renaming_playlist {
                    self.ui.new_playlist_name.pop();
                } else if self.ui.is_setting_alarm {
                    self.ui.alarm_input.pop();
                } else {
                    self.ui.search_query.pop();
                }
            }
            KeyCode::Enter if self.ui.input_focus || self.ui.is_renaming_playlist || self.ui.is_setting_alarm => {
                if self.ui.is_renaming_playlist {
                    let name = self.ui.new_playlist_name.clone();
                    if !name.trim().is_empty() {
                        self.ui.playlists[self.ui.current_playlist_idx].name = name;
                        let _ = self.ui.save_library();
                    }
                    self.ui.is_renaming_playlist = false;
                } else if self.ui.is_setting_alarm {
                    if let Some(track) = self.ui.alarm_track.take() {
                        let input = self.ui.alarm_input.trim();
                        let now = Local::now();
                        let is_repeat = self.ui.alarm_repeat_input;
                        let is_loop = self.ui.alarm_loop_input;
                        
                        let target_time = if input.contains('/') || (input.contains(':') && input.len() > 5) {
                            let mut dt_str = input.to_string();
                            if input.matches('/').count() == 1 { dt_str = format!("{}/{}", now.year(), input); }
                            NaiveDateTime::parse_from_str(&format!("{}:00", dt_str), "%Y/%m/%d %H:%M:%S")
                                .ok()
                                .and_then(|ndt| ndt.and_local_timezone(Local).single())
                                .unwrap_or(now)
                        } else if input.contains(':') {
                            let parts: Vec<&str> = input.split(':').collect();
                            if parts.len() == 2 {
                                let h: u32 = parts[0].parse().unwrap_or(0);
                                let m: u32 = parts[1].parse().unwrap_or(0);
                                if let Some(target) = NaiveTime::from_hms_opt(h, m, 0) {
                                    let target_dt = Local::now().date_naive().and_time(target).and_local_timezone(Local).single().unwrap();
                                    if target_dt <= now { target_dt + chrono::Duration::days(1) } else { target_dt }
                                } else { now }
                            } else { now }
                        } else {
                            let mins: i64 = input.parse().unwrap_or(0);
                            now + chrono::Duration::minutes(mins)
                        };

                        if target_time > now {
                            self.ui.alarms.push(crate::ui::Alarm {
                                target_time,
                                track,
                                is_active: true,
                                repeat_daily: is_repeat,
                                infinite_loop: is_loop,
                            });
                            let _ = self.ui.save_library();
                        }
                    }
                    self.ui.is_setting_alarm = false;
                    self.ui.alarm_input.clear();
                    self.ui.alarm_repeat_input = false;
                    self.ui.alarm_loop_input = false;
                } else {
                    if !self.ui.search_query.is_empty() {
                        self.ui.is_loading = true;
                        self.ui.search_performed = true;
                        self.ui.search_offset = 0;
                        self.ui.view_mode = crate::ui::ViewMode::Search;
                        self.ui.input_focus = false;
                    }
                }
            }

            // Tab Navigation - Vim keys (h/l) and Arrow keys
            KeyCode::Char('h') | KeyCode::Left if !self.ui.input_focus && !self.ui.is_renaming_playlist && !self.ui.is_setting_alarm && self.ui.focused_pane == crate::ui::Pane::Main => {
                self.ui.current_tab = match self.ui.current_tab {
                    crate::ui::Tab::Home => crate::ui::Tab::Library,
                    crate::ui::Tab::Search => crate::ui::Tab::Home,
                    crate::ui::Tab::Library => crate::ui::Tab::Search,
                };
                if self.ui.current_tab == crate::ui::Tab::Library {
                    self.ui.view_mode = crate::ui::ViewMode::PlaylistList;
                }
            }
            KeyCode::Char('l') | KeyCode::Right if !self.ui.input_focus && !self.ui.is_renaming_playlist && !self.ui.is_setting_alarm && self.ui.focused_pane == crate::ui::Pane::Main => {
                self.ui.current_tab = match self.ui.current_tab {
                    crate::ui::Tab::Home => crate::ui::Tab::Search,
                    crate::ui::Tab::Search => crate::ui::Tab::Library,
                    crate::ui::Tab::Library => crate::ui::Tab::Home,
                };
                if self.ui.current_tab == crate::ui::Tab::Library {
                    self.ui.view_mode = crate::ui::ViewMode::PlaylistList;
                }
            }

            KeyCode::Char('j') | KeyCode::Down if !self.ui.input_focus && !self.ui.is_renaming_playlist && !self.ui.is_setting_alarm => {
                match self.ui.focused_pane {
                    crate::ui::Pane::Info => {
                        let total = self.ui.search_history.len() + self.ui.alarms.len();
                        let i = match self.ui.history_state.selected() {
                            Some(i) => (i + 1) % total.max(1),
                            None => 0,
                        };
                        self.ui.history_state.select(Some(i));
                        self.ui.alarm_state.select(Some(i));
                    }
                    crate::ui::Pane::Sidebar => {
                        let i = match self.ui.playlist_list_state.selected() {
                            Some(i) => (i + 1) % self.ui.playlists.len(),
                            None => 0,
                        };
                        self.ui.playlist_list_state.select(Some(i));
                        self.ui.current_playlist_idx = i;
                    }
                    crate::ui::Pane::Main => {
                        match self.ui.current_tab {
                            crate::ui::Tab::Search => {
                                let current_idx = self.ui.selected_index;
                                let total_results = self.ui.search_results.len();
                                if current_idx >= total_results.saturating_sub(1) && total_results > 0 {
                                    self.load_more().await?;
                                } else {
                                    self.ui.move_selection_down();
                                }
                            }
                            crate::ui::Tab::Library => {
                                match self.ui.view_mode {
                                    crate::ui::ViewMode::PlaylistList => {
                                        let i = match self.ui.playlist_list_state.selected() {
                                            Some(i) => (i + 1) % self.ui.playlists.len(),
                                            None => 0,
                                        };
                                        self.ui.playlist_list_state.select(Some(i));
                                    }
                                    crate::ui::ViewMode::PlaylistDetail => {
                                        let playlist = &self.ui.playlists[self.ui.current_playlist_idx];
                                        if !playlist.tracks.is_empty() {
                                            let i = match self.ui.playlist_state.selected() {
                                                Some(i) => (i + 1) % playlist.tracks.len(),
                                                None => 0,
                                            };
                                            self.ui.playlist_state.select(Some(i));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up if !self.ui.input_focus && !self.ui.is_renaming_playlist && !self.ui.is_setting_alarm => {
                match self.ui.focused_pane {
                    crate::ui::Pane::Info => {
                        let total = self.ui.search_history.len() + self.ui.alarms.len();
                        let i = match self.ui.history_state.selected() {
                            Some(i) => if i == 0 { total.saturating_sub(1) } else { i - 1 },
                            None => 0,
                        };
                        self.ui.history_state.select(Some(i));
                        self.ui.alarm_state.select(Some(i));
                    }
                    crate::ui::Pane::Sidebar => {
                        let i = match self.ui.playlist_list_state.selected() {
                            Some(i) => if i == 0 { self.ui.playlists.len() - 1 } else { i - 1 },
                            None => 0,
                        };
                        self.ui.playlist_list_state.select(Some(i));
                        self.ui.current_playlist_idx = i;
                    }
                    crate::ui::Pane::Main => {
                        match self.ui.current_tab {
                            crate::ui::Tab::Search => self.ui.move_selection_up(),
                            crate::ui::Tab::Library => {
                                match self.ui.view_mode {
                                    crate::ui::ViewMode::PlaylistList => {
                                        let i = match self.ui.playlist_list_state.selected() {
                                            Some(i) => if i == 0 { self.ui.playlists.len() - 1 } else { i - 1 },
                                            None => 0,
                                        };
                                        self.ui.playlist_list_state.select(Some(i));
                                    }
                                    crate::ui::ViewMode::PlaylistDetail => {
                                        let playlist = &self.ui.playlists[self.ui.current_playlist_idx];
                                        if !playlist.tracks.is_empty() {
                                            let i = match self.ui.playlist_state.selected() {
                                                Some(i) => if i == 0 { playlist.tracks.len() - 1 } else { i - 1 },
                                                None => 0,
                                            };
                                            self.ui.playlist_state.select(Some(i));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') if !self.ui.input_focus && !self.ui.is_renaming_playlist && !self.ui.is_setting_alarm => {
                match self.ui.focused_pane {
                    crate::ui::Pane::Info => {
                        if let Some(i) = self.ui.history_state.selected() {
                            if i < self.ui.search_history.len() {
                                if let Some(query) = self.ui.search_history.get(i).cloned() {
                                    self.ui.search_query = query;
                                    self.ui.search_offset = 0;
                                    self.ui.is_loading = true;
                                    self.ui.search_performed = true;
                                    self.ui.current_tab = crate::ui::Tab::Search;
                                    self.ui.focused_pane = crate::ui::Pane::Main;
                                }
                            }
                        }
                    }
                    crate::ui::Pane::Sidebar => {
                        self.ui.focused_pane = crate::ui::Pane::Main;
                        self.ui.current_tab = crate::ui::Tab::Library;
                        self.ui.view_mode = crate::ui::ViewMode::PlaylistDetail;
                    }
                    crate::ui::Pane::Main => {
                        match self.ui.current_tab {
                            crate::ui::Tab::Library => {
                                if self.ui.view_mode == crate::ui::ViewMode::PlaylistList {
                                    if let Some(i) = self.ui.playlist_list_state.selected() {
                                        self.ui.current_playlist_idx = i;
                                        self.ui.view_mode = crate::ui::ViewMode::PlaylistDetail;
                                        self.ui.playlist_state.select(Some(0));
                                    }
                                } else {
                                    let playlist = &self.ui.playlists[self.ui.current_playlist_idx];
                                    if let Some(track) = playlist.tracks.get(self.ui.playlist_state.selected().unwrap_or(0)).cloned() {
                                        let _ = self.player.play(track.clone()).await;
                                        self.ui.current_track = Some(track.clone());
                                        self.ui.is_playing = true;
                                        
                                        // Apply current repeat mode
                                        let mode_str = match self.ui.repeat_mode {
                                            crate::ui::RepeatMode::One => "One",
                                            crate::ui::RepeatMode::All => "All",
                                            crate::ui::RepeatMode::Off => "Off",
                                        };
                                        self.player.set_repeat_mode(mode_str);

                                        self.ui.play_history.insert(0, track);
                                        if self.ui.play_history.len() > 20 { self.ui.play_history.pop(); }
                                    }
                                }
                            }
                            crate::ui::Tab::Search => {
                                if let Some(track) = self.ui.get_selected_track() {
                                    if track.title.contains("[Playlist]") || track.url.contains("playlist?list=") {
                                        self.ui.is_loading = true;
                                        self.ui.search_query = track.url.clone();
                                        self.ui.search_mode = crate::ui::SearchMode::Track;
                                        self.ui.search_offset = 0;
                                    } else {
                                        let _ = self.player.play(track.clone()).await;
                                        self.ui.current_track = Some(track.clone());
                                        self.ui.is_playing = true;

                                        // Apply current repeat mode
                                        let mode_str = match self.ui.repeat_mode {
                                            crate::ui::RepeatMode::One => "One",
                                            crate::ui::RepeatMode::All => "All",
                                            crate::ui::RepeatMode::Off => "Off",
                                        };
                                        self.player.set_repeat_mode(mode_str);

                                        self.ui.play_history.insert(0, track);
                                        if self.ui.play_history.len() > 20 { self.ui.play_history.pop(); }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Playlist Management
            KeyCode::Char('a') if !self.ui.input_focus => {
                if let Some(track) = self.ui.get_selected_track() {
                    let mut added = false;
                    {
                        let playlist = &mut self.ui.playlists[self.ui.current_playlist_idx];
                        if !playlist.tracks.contains(&track) {
                            playlist.tracks.push(track);
                            added = true;
                        }
                    }
                    if added { let _ = self.ui.save_library(); }
                }
            }
            KeyCode::Char('n') if !self.ui.input_focus => {
                let name = format!("Playlist {}", self.ui.playlists.len() + 1);
                self.ui.playlists.push(crate::ui::Playlist { name, tracks: Vec::new() });
                let _ = self.ui.save_library();
            }
            KeyCode::Char('r') if !self.ui.input_focus => {
                if self.ui.current_tab == crate::ui::Tab::Library && self.ui.view_mode == crate::ui::ViewMode::PlaylistList {
                    if let Some(i) = self.ui.playlist_list_state.selected() {
                        self.ui.current_playlist_idx = i;
                        self.ui.new_playlist_name = self.ui.playlists[i].name.clone();
                        self.ui.is_renaming_playlist = true;
                    }
                } else {
                    self.ui.repeat_mode = match self.ui.repeat_mode {
                        crate::ui::RepeatMode::Off => crate::ui::RepeatMode::One,
                        crate::ui::RepeatMode::One => crate::ui::RepeatMode::All,
                        crate::ui::RepeatMode::All => crate::ui::RepeatMode::Off,
                    };
                    let mode_str = match self.ui.repeat_mode {
                        crate::ui::RepeatMode::One => "One",
                        crate::ui::RepeatMode::All => "All",
                        crate::ui::RepeatMode::Off => "Off",
                    };
                    self.player.set_repeat_mode(mode_str);
                }
            }
            KeyCode::Char('d') if !self.ui.input_focus => {
                match self.ui.focused_pane {
                    crate::ui::Pane::Info => {
                        if let Some(i) = self.ui.alarm_state.selected() {
                            if i >= self.ui.search_history.len() {
                                let alarm_idx = i - self.ui.search_history.len();
                                if alarm_idx < self.ui.alarms.len() {
                                    self.ui.alarms.remove(alarm_idx);
                                }
                            }
                        }
                    }
                    crate::ui::Pane::Sidebar => {
                        if self.ui.playlists.len() > 1 {
                            if let Some(i) = self.ui.playlist_list_state.selected() {
                                self.ui.playlists.remove(i);
                                self.ui.current_playlist_idx = 0;
                                self.ui.playlist_list_state.select(Some(0));
                                let _ = self.ui.save_library();
                            }
                        }
                    }
                    crate::ui::Pane::Main => {
                        match self.ui.view_mode {
                            crate::ui::ViewMode::PlaylistDetail => {
                                let mut changed = false;
                                {
                                    let playlist = &mut self.ui.playlists[self.ui.current_playlist_idx];
                                    if let Some(i) = self.ui.playlist_state.selected() {
                                        if i < playlist.tracks.len() {
                                            playlist.tracks.remove(i);
                                            changed = true;
                                            if playlist.tracks.is_empty() { self.ui.playlist_state.select(None); }
                                            else { let next_i = i.min(playlist.tracks.len() - 1); self.ui.playlist_state.select(Some(next_i)); }
                                        }
                                    }
                                }
                                if changed { let _ = self.ui.save_library(); }
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // Cache / Alarm
            KeyCode::Char('c') if !self.ui.input_focus => {
                if let Some(track) = self.ui.get_selected_track() { let _ = self.player.download(track).await; }
            }
            KeyCode::Char('C') if !self.ui.input_focus => {
                if let Some(track) = self.ui.get_selected_track() { let _ = self.player.uncache(&track); }
            }
            KeyCode::Char('A') if !self.ui.input_focus => {
                if let Some(track) = self.ui.get_selected_track() {
                    self.ui.alarm_track = Some(track);
                    self.ui.alarm_input.clear();
                    self.ui.is_setting_alarm = true;
                }
            }

            // Pause/Resume with p
            KeyCode::Char('p') if !self.ui.input_focus => {
                if self.ui.is_playing { self.player.pause(); self.ui.is_playing = false; } 
                else { self.player.resume(); self.ui.is_playing = true; }
            }
            KeyCode::Char('s') if !self.ui.input_focus => {
                self.player.stop(); self.ui.is_playing = false; self.ui.current_track = None;
            }
            KeyCode::Tab => {
                if self.ui.is_setting_alarm {
                    if self.ui.alarm_repeat_input && !self.ui.alarm_loop_input {
                        self.ui.alarm_repeat_input = false;
                        self.ui.alarm_loop_input = true;
                    } else if !self.ui.alarm_repeat_input && self.ui.alarm_loop_input {
                        self.ui.alarm_repeat_input = true;
                        self.ui.alarm_loop_input = true;
                    } else if self.ui.alarm_repeat_input && self.ui.alarm_loop_input {
                        self.ui.alarm_repeat_input = false;
                        self.ui.alarm_loop_input = false;
                    } else {
                        self.ui.alarm_repeat_input = true;
                        self.ui.alarm_loop_input = false;
                    }
                } else if self.ui.input_focus {
                    self.ui.search_mode = match self.ui.search_mode {
                        crate::ui::SearchMode::Track => crate::ui::SearchMode::Playlist,
                        crate::ui::SearchMode::Playlist => crate::ui::SearchMode::Track,
                    };
                } else {
                    self.ui.focused_pane = match self.ui.focused_pane {
                        crate::ui::Pane::Sidebar => crate::ui::Pane::Main,
                        crate::ui::Pane::Main => crate::ui::Pane::Info,
                        crate::ui::Pane::Info => crate::ui::Pane::Sidebar,
                    };
                }
            }

            // Volume
            KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char(']') if !self.ui.input_focus => {
                if self.ui.volume < 100 { self.ui.volume = (self.ui.volume + 5).min(100); self.player.set_volume(self.ui.volume); }
            }
            KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Char('[') if !self.ui.input_focus => {
                if self.ui.volume > 0 { self.ui.volume = self.ui.volume.saturating_sub(5); self.player.set_volume(self.ui.volume); }
            }
            KeyCode::Char(c) if c.is_ascii_digit() && !self.ui.input_focus => {
                let val = c.to_digit(10).unwrap_or(0);
                self.ui.volume = if val == 0 { 100 } else { (val * 10) as u8 };
                self.player.set_volume(self.ui.volume);
            }
            KeyCode::Char('m') if !self.ui.input_focus => {
                self.ui.volume = if self.ui.volume > 0 { 0 } else { 50 };
                self.player.set_volume(self.ui.volume);
            }

            // Focus search with /
            KeyCode::Char('/') if !self.ui.input_focus => {
                self.ui.current_tab = crate::ui::Tab::Search;
                self.ui.input_focus = true;
                self.ui.search_query.clear();
            }

            _ => {}
        }
        Ok(true)
    }

    async fn perform_search(&mut self) -> Result<()> {
        let query = self.ui.search_query.clone();
        let platform_filter = self.ui.platform_filter.clone();
        let offset = self.ui.search_offset;
        let is_playlist_search = self.ui.search_mode == crate::ui::SearchMode::Playlist;

        if offset == 0 {
            self.ui.search_results.clear();
            self.ui.selected_index = 0;
            self.ui.list_state.select(Some(0));
            self.ui.view_mode = crate::ui::ViewMode::Search;
            if !self.ui.search_history.contains(&query) {
                self.ui.search_history.insert(0, query.clone());
                if self.ui.search_history.len() > 20 { self.ui.search_history.pop(); }
                let _ = self.ui.save_library();
            }
        }

        for provider in &self.search_providers {
            if platform_filter == "All" || provider.platform_name() == platform_filter {
                match provider.search(&query, 10, offset, is_playlist_search).await {
                    Ok(results) => { self.ui.search_results.extend(results); }
                    Err(e) => { log::warn!("Search failed for {}: {}", provider.platform_name(), e); }
                }
            }
        }
        self.ui.is_loading = false;
        Ok(())
    }

    async fn load_more(&mut self) -> Result<()> {
        if !self.ui.is_loading && !self.ui.search_query.is_empty() {
            self.ui.search_offset += 10;
            self.ui.is_loading = true;
        }
        Ok(())
    }

    fn cycle_platform_filter(&mut self) {
        let platforms = vec!["All", "YouTube"];
        let current_idx = platforms.iter().position(|p| p == &self.ui.platform_filter).unwrap_or(0);
        let next_idx = (current_idx + 1) % platforms.len();
        self.ui.platform_filter = platforms[next_idx].to_string();
    }

    async fn play_next(&mut self) -> Result<()> {
        let next_track = match self.ui.view_mode {
            crate::ui::ViewMode::Search => {
                let current = self.ui.selected_index;
                if current + 1 < self.ui.search_results.len() {
                    self.ui.selected_index += 1;
                    self.ui.list_state.select(Some(self.ui.selected_index));
                    self.ui.search_results.get(self.ui.selected_index).cloned()
                } else if !self.ui.search_results.is_empty() {
                    self.ui.selected_index = 0;
                    self.ui.list_state.select(Some(0));
                    self.ui.search_results.get(0).cloned()
                } else { None }
            }
            crate::ui::ViewMode::PlaylistDetail => {
                let playlist = &self.ui.playlists[self.ui.current_playlist_idx];
                let current = self.ui.playlist_state.selected().unwrap_or(0);
                if !playlist.tracks.is_empty() {
                    let next_idx = (current + 1) % playlist.tracks.len();
                    self.ui.playlist_state.select(Some(next_idx));
                    playlist.tracks.get(next_idx).cloned()
                } else { None }
            }
            _ => None,
        };

        if let Some(track) = next_track {
            let _ = self.player.play(track.clone()).await;
            self.ui.current_track = Some(track.clone());
            self.ui.is_playing = true;

            let mode_str = match self.ui.repeat_mode {
                crate::ui::RepeatMode::One => "One",
                crate::ui::RepeatMode::All => "All",
                crate::ui::RepeatMode::Off => "Off",
            };
            self.player.set_repeat_mode(mode_str);

            self.ui.play_history.insert(0, track);
            if self.ui.play_history.len() > 20 { self.ui.play_history.pop(); }
        }
        Ok(())
    }
}
