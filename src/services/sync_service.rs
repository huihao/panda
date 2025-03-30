use std::sync::Arc;
use anyhow::Result;
use log::{info, error};
use tokio::time::{sleep, Duration};

use crate::services::RssService;

/// Service for managing feed synchronization
pub struct SyncService {
    rss_service: Arc<RssService>,
    auto_sync: bool,
    sync_interval: u64,
}

impl SyncService {
    /// Creates a new sync service
    pub fn new(rss_service: Arc<RssService>) -> Self {
        Self {
            rss_service,
            auto_sync: true,
            sync_interval: 3600, // Default to 1 hour
        }
    }
    
    /// Sets whether auto-sync is enabled
    pub fn set_auto_sync(&mut self, enabled: bool) {
        self.auto_sync = enabled;
    }
    
    /// Sets the sync interval in seconds
    pub fn set_sync_interval(&mut self, interval: u64) {
        self.sync_interval = interval;
    }
    
    /// Starts the sync service
    pub async fn start(&self) -> Result<()> {
        if !self.auto_sync {
            return Ok(());
        }
        
        loop {
            match self.rss_service.fetch_all_feeds().await {
                Ok(_) => {
                    info!("Feed sync completed successfully");
                }
                Err(e) => {
                    error!("Feed sync failed: {}", e);
                }
            }
            
            sleep(Duration::from_secs(self.sync_interval)).await;
        }
    }
    
    /// Synchronizes all feeds immediately
    pub async fn sync_all_feeds(&self) -> Result<()> {
        self.rss_service.fetch_all_feeds().await
    }
} 