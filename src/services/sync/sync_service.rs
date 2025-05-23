use std::sync::Arc;
use anyhow::Result;
use crate::services::rss::RssService;

pub struct SyncService {
    rss_service: Arc<RssService>,
}

impl SyncService {
    pub fn new(rss_service: Arc<RssService>) -> Self {
        Self {
            rss_service,
        }
    }

    pub async fn sync_all(&self) -> Result<()> {
        self.rss_service.sync_all_feeds().await
    }

    pub async fn sync_feed(&self, feed_id: &str) -> Result<()> {
        // Create a proper FeedId from the string
        let feed_id = crate::models::feed::FeedId(feed_id.to_string());
        self.rss_service.sync_feed(&feed_id).await
    }
}