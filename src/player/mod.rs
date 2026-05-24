use std::process::{Command as StdCommand, Child, Stdio};
use std::io::Write;
use std::os::unix::net::UnixStream;
use anyhow::Result;
use std::fs;
use std::time::Duration;
use std::thread;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub url: String,
    pub title: String,
    pub platform: String,
    pub duration: Option<u64>,
}

pub struct Player {
    child: Option<Child>,
    ipc_path: String,
    cache_dir: std::path::PathBuf,
}

impl Player {
    pub fn new() -> Self {
        let ipc_path = format!("/tmp/melody-mpv-{}.sock", std::process::id());
        let cache_dir = std::path::PathBuf::from("cache");
        let _ = fs::create_dir_all(&cache_dir);
        
        Player {
            child: None,
            ipc_path,
            cache_dir,
        }
    }

    pub fn get_cache_path(&self, track: &Track) -> std::path::PathBuf {
        // Use a simple hash or sanitize the title for filename
        let safe_title = track.title.chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .collect::<String>()
            .replace(" ", "_");
        
        // We'll use the URL hash or ID to keep it unique
        let id = track.url.split('=').last().unwrap_or("unknown");
        self.cache_dir.join(format!("{}_{}.mp3", safe_title, id))
    }

    pub fn is_cached(&self, track: &Track) -> bool {
        self.get_cache_path(track).exists()
    }

    pub async fn download(&self, track: Track) -> Result<()> {
        let path = self.get_cache_path(&track);
        if path.exists() {
            return Ok(());
        }

        let mut child = StdCommand::new("yt-dlp")
            .arg("-x")
            .arg("--audio-format")
            .arg("mp3")
            .arg("-o")
            .arg(&path)
            .arg(&track.url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // We don't want to block the UI, but we should let it run in background
        tokio::spawn(async move {
            let _ = child.wait();
        });

        Ok(())
    }

    pub fn uncache(&self, track: &Track) -> Result<()> {
        let path = self.get_cache_path(track);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn ensure_mpv(&mut self) -> Result<()> {
        let is_alive = if let Some(child) = &mut self.child {
            child.try_wait()?.is_none() && UnixStream::connect(&self.ipc_path).is_ok()
        } else {
            false
        };

        if !is_alive {
            log::info!("Starting new mpv instance...");
            self.kill(); // Ensure everything is cleaned up
            
            let child = StdCommand::new("mpv")
                .arg("--no-video")
                .arg("--idle")
                .arg(format!("--input-ipc-server={}", self.ipc_path))
                .arg("--ytdl-format=bestaudio")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;

            self.child = Some(child);
            
            // Wait for socket to be ready
            for _ in 0..20 {
                if fs::metadata(&self.ipc_path).is_ok() && UnixStream::connect(&self.ipc_path).is_ok() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        Ok(())
    }

    pub async fn play(&mut self, track: Track) -> Result<()> {
        log::info!("Playing track: {}", track.title);
        self.ensure_mpv()?;
        
        let source = if self.is_cached(&track) {
            let path = self.get_cache_path(&track);
            log::info!("Using cached file: {:?}", path);
            path.to_string_lossy().to_string()
        } else {
            log::info!("Streaming from URL: {}", track.url);
            track.url.clone()
        };

        let cmd = format!("{{ \"command\": [\"loadfile\", \"{}\", \"replace\"] }}\n", source);
        self.send_command(&cmd)?;
        
        // Brief sleep to let mpv start loading before we query its state
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        Ok(())
    }

    pub fn pause(&mut self) {
        let _ = self.send_command("set pause yes\n");
    }

    pub fn resume(&mut self) {
        let _ = self.send_command("set pause no\n");
    }

    pub fn stop(&mut self) {
        // Just stop playback without killing the process for faster restart
        let _ = self.send_command("stop\n");
    }

    pub fn set_volume(&mut self, volume: u8) {
        let _ = self.send_command(&format!("set volume {}\n", volume));
    }

    pub fn set_repeat_mode(&mut self, mode: &str) {
        let val = match mode {
            "One" => "inf",
            _ => "no",
        };
        let _ = self.send_command(&format!("{{ \"command\": [\"set_property\", \"loop-file\", \"{}\"] }}\n", val));
    }

    pub fn get_property(&self, prop: &str) -> Result<String> {
        if fs::metadata(&self.ipc_path).is_ok() {
            let mut stream = UnixStream::connect(&self.ipc_path)?;
            stream.set_read_timeout(Some(Duration::from_millis(500)))?;
            
            let cmd = format!("{{ \"command\": [\"get_property\", \"{}\"] }}\n", prop);
            let _ = stream.write_all(cmd.as_bytes());
            
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stream);
            for line in reader.lines() {
                if let Ok(l) = line {
                    // We look for the response that contains "data" field
                    if l.contains("\"data\":") {
                        return Ok(l);
                    }
                } else {
                    break;
                }
            }
            anyhow::bail!("No data in response")
        } else {
            anyhow::bail!("IPC not connected")
        }
    }

    pub fn is_idle(&self) -> bool {
        match self.get_property("idle-active") {
            Ok(resp) => resp.contains("true"),
            Err(_) => false, // Assume not idle on error to prevent premature stopping
        }
    }

    fn send_command(&self, cmd: &str) -> Result<()> {
        let mut stream = UnixStream::connect(&self.ipc_path)?;
        stream.write_all(cmd.as_bytes())?;
        Ok(())
    }

    pub fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = self.send_command("quit\n");
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = fs::remove_file(&self.ipc_path);
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.kill();
    }
}
