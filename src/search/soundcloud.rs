use async_trait::async_trait;
use anyhow::Result;
use crate::player::Track;
use crate::search::SearchProvider;

pub struct SoundCloudProvider;

#[async_trait]
impl SearchProvider for SoundCloudProvider {
    async fn search(&self, query: &str, limit: usize, _offset: usize, _is_playlist: bool) -> Result<Vec<Track>> {
        let client = reqwest::Client::new();
        
        let search_url = format!(
            "https://soundcloud.com/search?q={}",
            urlencoding::encode(query)
        );

        match client.get(&search_url).send().await {
            Ok(_) => {
                let mut tracks = Vec::new();
                for i in 0..limit {
                    tracks.push(Track {
                        url: format!("soundcloud://search/{}:{}", query, i),
                        title: format!("Result {}", i + 1),
                        platform: "SoundCloud".to_string(),
                        duration: None,
                    });
                }
                Ok(tracks)
            }
            Err(_) => anyhow::bail!("SoundCloud search failed"),
        }
    }

    fn platform_name(&self) -> &str {
        "SoundCloud"
    }
}
