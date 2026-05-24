use async_trait::async_trait;
use anyhow::Result;
use crate::player::Track;
use crate::search::SearchProvider;
use tokio::process::Command;
use serde_json::Value;

pub struct YouTubeProvider;

#[async_trait]
impl SearchProvider for YouTubeProvider {
    async fn search(&self, query: &str, limit: usize, _offset: usize, is_playlist: bool) -> Result<Vec<Track>> {
        let mut cmd = Command::new("yt-dlp");

        if query.contains("playlist?list=") || query.contains("&list=") {
            // Expanding a direct playlist URL
            cmd.arg(query)
               .arg("--dump-json")
               .arg("--flat-playlist")
               .arg(format!("--playlist-end={}", limit))
               .arg("-q");
        } else if is_playlist {
            // Search specifically for PLAYLISTS using YouTube's filter parameter
            let search_url = format!(
                "https://www.youtube.com/results?search_query={}&sp=EgIQAw%253D%253D",
                urlencoding::encode(query)
            );
            cmd.arg(search_url)
               .arg("--dump-json")
               .arg("--flat-playlist")
               .arg(format!("--playlist-end={}", limit))
               .arg("-q");
        } else {
            // Normal track search
            cmd.arg(format!("ytsearch{}:{}", limit, query))
               .arg("--dump-json")
               .arg("--flat-playlist")
               .arg("--no-playlist")
               .arg("-q");
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("YouTube search failed: {}", stderr);
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut tracks = Vec::new();
        
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(line) {
                Ok(json) => {
                    let title = json["title"].as_str().unwrap_or("Unknown");
                    let webpage_url = json["webpage_url"].as_str();
                    let url = json["url"].as_str();
                    let duration = json["duration"].as_i64();
                    
                    let final_url = webpage_url.or(url).unwrap_or("");
                    let entry_type = json["_type"].as_str().unwrap_or("video");
                    
                    if !final_url.is_empty() && !title.is_empty() {
                        let display_title = if entry_type == "playlist" || entry_type == "url_transparent" {
                            format!("󰲒 [Playlist] {}", title)
                        } else {
                            title.to_string()
                        };

                        tracks.push(Track {
                            url: final_url.to_string(),
                            title: display_title,
                            platform: "YouTube".to_string(),
                            duration: duration.map(|d| d as u64),
                        });
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(tracks.into_iter().take(limit).collect())
    }

    fn platform_name(&self) -> &str {
        "YouTube"
    }
}
