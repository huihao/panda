use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

use crate::models::article::{Article, ArticleId};
use crate::models::feed::FeedId;
use crate::models::category::CategoryId;

/// Trait defining the interface for article repository implementations
#[async_trait]
pub trait ArticleRepository: Send + Sync {
    /// Saves an article to the repository
    async fn save_article(&self, article: &Article) -> Result<()>;
    
    /// Retrieves an article by its ID
    async fn get_article(&self, id: &ArticleId) -> Result<Option<Article>>;
    
    /// Retrieves an article by its URL
    async fn get_article_by_url(&self, url: &str) -> Result<Option<Article>>;
    
    /// Retrieves all articles from the repository
    async fn get_all_articles(&self) -> Result<Vec<Article>>;
    
    /// Retrieves all articles from a specific feed
    async fn get_articles_by_feed(&self, feed_id: &FeedId) -> Result<Vec<Article>>;
    
    /// Retrieves all articles from a specific category
    async fn get_articles_by_category(&self, category_id: &CategoryId) -> Result<Vec<Article>>;
    
    /// Retrieves all unread articles
    async fn get_unread_articles(&self) -> Result<Vec<Article>>;
    
    /// Retrieves all favorite articles
    async fn get_favorite_articles(&self) -> Result<Vec<Article>>;
    
    /// Updates an existing article
    async fn update_article(&self, article: &Article) -> Result<()>;
    
    /// Deletes an article by its ID
    async fn delete_article(&self, id: &ArticleId) -> Result<()>;
    
    /// Adds a tag to an article
    async fn add_tag(&self, article_id: &ArticleId, tag: &str) -> Result<()>;
    
    /// Removes a tag from an article
    async fn remove_tag(&self, article_id: &ArticleId, tag: &str) -> Result<()>;
    
    /// Retrieves all tags associated with an article
    async fn get_article_tags(&self, article_id: &ArticleId) -> Result<Vec<String>>;
    
    /// Retrieves all articles with a specific tag
    async fn get_articles_by_tag(&self, tag: &str) -> Result<Vec<Article>>;
    
    /// Retrieves articles published within the given date range
    async fn get_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>>;
    
    /// Searches for articles matching the given query
    async fn search_articles(&self, query: &str) -> Result<Vec<Article>>;
}