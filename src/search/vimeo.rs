use async_trait::async_trait;
use anyhow::Result;
use crate::player::Track;
use crate::search::SearchProvider;

pub struct VimeoProvider;

#[async_trait]
impl SearchProvider for VimeoProvider {
    async fn search(&self, _query: &str, _limit: usize, _offset: usize, _is_playlist: bool) -> Result<Vec<Track>> {
        Ok(Vec::new())
    }

    fn platform_name(&self) -> &str {
        "Vimeo"
    }
}
