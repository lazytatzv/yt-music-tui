use anyhow::Result;
use crate::player::Track;

#[async_trait::async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &str, limit: usize, offset: usize, is_playlist: bool) -> Result<Vec<Track>>;
    fn platform_name(&self) -> &str;
}

pub mod youtube;
pub mod soundcloud;
pub mod bandcamp;
pub mod dailymotion;
pub mod vimeo;
pub mod twitch;

