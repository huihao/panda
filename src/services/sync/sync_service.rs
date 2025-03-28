use anyhow::Result;
use log::{info, error};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::models::feed::{Feed, FeedStatus};
use crate::data::repositories::{FeedRepository, ArticleRepository};
use crate::services::rss::RssService;

/// Service for handling feed synchronization
pub struct SyncService {
    feed_repository: Box<dyn FeedRepository>,
    article_repository: Box<dyn ArticleRepository>,
    rss_service: Arc<RssService>,
}

impl SyncService {
    /// Creates a new sync service
    pub fn new(
        feed_repository: Box<dyn FeedRepository>,
        article_repository: Box<dyn ArticleRepository>,
        rss_service: Arc<RssService>,
    ) -> Self {
        Self {
            feed_repository,
            article_repository,
            rss_service,
        }
    }

    /// Starts the feed sync service
    pub async fn start(&self) -> Result<()> {
        info!("Starting feed sync service");
        loop {
            self.sync_feeds().await?;
            sleep(Duration::from_secs(300)).await; // Sleep for 5 minutes
        }
    }

    /// Syncs all active feeds
    pub async fn sync_feeds(&self) -> Result<()> {
        info!("Syncing feeds");
        let feeds = self.feed_repository.get_active_feeds()?;

        for feed in feeds {
            if let Err(e) = self.sync_feed(&feed).await {
                error!("Failed to sync feed {}: {}", feed.url, e);
                self.feed_repository.update_feed_status(&feed.id, FeedStatus::Error, Some(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Syncs a single feed
    async fn sync_feed(&self, feed: &Feed) -> Result<()> {
        info!("Syncing feed: {}", feed.url);
        self.rss_service.sync_feed(feed).await?;
        Ok(())
    }

    /// Performs an immediate sync of all feeds
    pub async fn sync_now(&self) -> Result<()> {
        info!("Triggering immediate feed sync");
        self.sync_feeds().await
    }
}