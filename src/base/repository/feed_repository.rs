use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

use crate::models::feed::{Feed, FeedId};
use crate::models::category::CategoryId;

/// Trait defining the interface for feed repository implementations
#[async_trait]
pub trait FeedRepository: Send + Sync {
    /// Saves a feed to the repository
    async fn save_feed(&self, feed: &Feed) -> Result<()>;
    
    /// Retrieves a feed by its ID
    async fn get_feed_by_id(&self, id: &FeedId) -> Result<Option<Feed>>;
    
    /// Retrieves a feed by its URL
    async fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>>;
    
    /// Retrieves all feeds from the repository
    async fn get_all_feeds(&self) -> Result<Vec<Feed>>;
    
    /// Retrieves all feeds from a specific category
    async fn get_feeds_by_category(&self, category_id: &CategoryId) -> Result<Vec<Feed>>;
    
    /// Retrieves all enabled feeds
    async fn get_enabled_feeds(&self) -> Result<Vec<Feed>>;
    
    /// Retrieves all feeds that need to be updated
    async fn get_feeds_to_update(&self) -> Result<Vec<Feed>>;
    
    /// Updates an existing feed
    async fn update_feed(&self, feed: &Feed) -> Result<()>;
    
    /// Deletes a feed by its ID
    async fn delete_feed(&self, id: &FeedId) -> Result<()>;
    
    /// Searches for feeds matching the given query
    async fn search_feeds(&self, query: &str) -> Result<Vec<Feed>>;
    
    /// Retrieves feeds created within the given date range
    async fn get_feeds_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Feed>>;
    
    /// Retrieves the most recently updated feeds
    async fn get_recently_updated_feeds(&self, limit: usize) -> Result<Vec<Feed>>;
    
    /// Retrieves the most frequently updated feeds
    async fn get_most_active_feeds(&self, limit: usize) -> Result<Vec<Feed>>;
}