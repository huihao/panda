use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::models::feed::{Feed, FeedId, CategoryId};

/// Trait defining the interface for feed repository implementations
pub trait FeedRepository: Send + Sync {
    /// Saves a feed to the repository
    fn save_feed(&self, feed: &Feed) -> Result<()>;
    
    /// Retrieves a feed by its ID
    fn get_feed_by_id(&self, id: &FeedId) -> Result<Option<Feed>>;
    
    /// Retrieves a feed by its URL
    fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>>;
    
    /// Retrieves all feeds from the repository
    fn get_all_feeds(&self) -> Result<Vec<Feed>>;
    
    /// Retrieves all feeds from a specific category
    fn get_feeds_by_category(&self, category_id: &str) -> Result<Vec<Feed>>;
    
    /// Retrieves all enabled feeds
    fn get_enabled_feeds(&self) -> Result<Vec<Feed>>;
    
    /// Retrieves all feeds that need to be updated
    fn get_feeds_to_update(&self) -> Result<Vec<Feed>>;
    
    /// Updates an existing feed
    fn update_feed(&self, feed: &Feed) -> Result<()>;
    
    /// Deletes a feed by its ID
    fn delete_feed(&self, id: &FeedId) -> Result<()>;
    
    /// Searches for feeds matching the given query
    fn search_feeds(&self, query: &str) -> Result<Vec<Feed>>;
    
    /// Retrieves feeds created within the given date range
    fn get_feeds_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Feed>>;
    
    /// Retrieves the most recently updated feeds
    fn get_recently_updated_feeds(&self, limit: usize) -> Result<Vec<Feed>>;
    
    /// Retrieves the most frequently updated feeds
    fn get_most_active_feeds(&self, limit: usize) -> Result<Vec<Feed>>;
} 