use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::models::article::{Article, ArticleId, FeedId, CategoryId, ReadStatus};

/// Trait defining the interface for article repository implementations
pub trait ArticleRepository: Send + Sync {
    /// Saves an article to the repository
    fn save_article(&self, article: &Article) -> Result<()>;
    
    /// Retrieves an article by its ID
    fn get_article(&self, id: &ArticleId) -> Result<Option<Article>>;
    
    /// Retrieves an article by its URL
    fn get_article_by_url(&self, url: &str) -> Result<Option<Article>>;
    
    /// Retrieves all articles from the repository
    fn get_all_articles(&self) -> Result<Vec<Article>>;
    
    /// Retrieves all articles from a specific feed
    fn get_articles_by_feed(&self, feed_id: &str) -> Result<Vec<Article>>;
    
    /// Retrieves all articles from a specific category
    fn get_articles_by_category(&self, category_id: &str) -> Result<Vec<Article>>;
    
    /// Retrieves all unread articles
    fn get_unread_articles(&self) -> Result<Vec<Article>>;
    
    /// Retrieves all favorite articles
    fn get_favorite_articles(&self) -> Result<Vec<Article>>;
    
    /// Updates an existing article
    fn update_article(&self, article: &Article) -> Result<()>;
    
    /// Deletes an article by its ID
    fn delete_article(&self, id: &ArticleId) -> Result<()>;
    
    /// Adds a tag to an article
    fn add_tag(&self, article_id: &ArticleId, tag: &str) -> Result<()>;
    
    /// Removes a tag from an article
    fn remove_tag(&self, article_id: &ArticleId, tag: &str) -> Result<()>;
    
    /// Retrieves all tags associated with an article
    fn get_article_tags(&self, article_id: &ArticleId) -> Result<Vec<String>>;
    
    /// Retrieves all articles with a specific tag
    fn get_articles_by_tag(&self, tag: &str) -> Result<Vec<Article>>;
    
    /// Retrieves articles published within the given date range
    fn get_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>>;
    
    /// Searches for articles matching the given query
    fn search_articles(&self, query: &str) -> Result<Vec<Article>>;
} 