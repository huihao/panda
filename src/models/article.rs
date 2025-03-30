use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use url::Url;
use std::fmt;
use std::str::FromStr;
use std::fmt::Display;
use uuid::Uuid;
use std::sync::Arc;
use anyhow::Result;

use super::feed::{FeedId, Tag};
use crate::models::{CategoryId};

/// A unique identifier for an article
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArticleId(pub String);

impl ArticleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for ArticleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier for a category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(pub String);

impl fmt::Display for CategoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The read status of an article
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReadStatus {
    Unread,
    Read,
    Archived,
}

impl Default for ReadStatus {
    fn default() -> Self {
        Self::Unread
    }
}

impl std::fmt::Display for ReadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadStatus::Unread => write!(f, "Unread"),
            ReadStatus::Read => write!(f, "Read"),
            ReadStatus::Archived => write!(f, "Archived"),
        }
    }
}

/// Represents an article from a feed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Article {
    /// The unique identifier of the article
    pub id: ArticleId,
    
    /// The ID of the feed this article belongs to
    pub feed_id: FeedId,
    
    /// The ID of the category this article belongs to (if any)
    pub category_id: Option<CategoryId>,
    
    /// The title of the article
    pub title: String,
    
    /// The URL of the article
    pub url: Url,
    
    /// The author of the article (if any)
    pub author: Option<String>,
    
    /// The content of the article (if any)
    pub content: Option<String>,
    
    /// The summary of the article (if any)
    pub summary: Option<String>,
    
    /// When the article was published
    pub published_at: Option<DateTime<Utc>>,
    
    /// The read status of the article
    pub read_status: ReadStatus,
    
    /// Whether the article is favorited
    pub is_favorited: bool,
    
    /// When the article was created in the database
    pub created_at: DateTime<Utc>,
    
    /// When the article was last updated in the database
    pub updated_at: DateTime<Utc>,
}

impl Article {
    /// Creates a new article with the given feed ID, title, and URL
    pub fn new(feed_id: FeedId, title: String, url: Url) -> Self {
        let now = Utc::now();
        Self {
            id: ArticleId(Uuid::new_v4().to_string()),
            feed_id,
            category_id: None,
            title,
            url,
            author: None,
            content: None,
            summary: None,
            published_at: None,
            read_status: ReadStatus::default(),
            is_favorited: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Sets the category ID of the article
    pub fn with_category(mut self, category_id: CategoryId) -> Self {
        self.category_id = Some(category_id);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the author of the article
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the content of the article
    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the summary of the article
    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets when the article was published
    pub fn with_published_at(mut self, published_at: DateTime<Utc>) -> Self {
        self.published_at = Some(published_at);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the read status of the article
    pub fn with_read_status(mut self, read_status: ReadStatus) -> Self {
        self.read_status = read_status;
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets whether the article is favorited
    pub fn with_favorited(mut self, is_favorited: bool) -> Self {
        self.is_favorited = is_favorited;
        self.updated_at = Utc::now();
        self
    }
    
    /// Updates the category ID of the article
    pub fn update_category(&mut self, category_id: Option<CategoryId>) {
        self.category_id = category_id;
        self.updated_at = Utc::now();
    }
    
    /// Updates the author of the article
    pub fn update_author(&mut self, author: Option<String>) {
        self.author = author;
        self.updated_at = Utc::now();
    }
    
    /// Updates the content of the article
    pub fn update_content(&mut self, content: Option<String>) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    
    /// Updates the summary of the article
    pub fn update_summary(&mut self, summary: Option<String>) {
        self.summary = summary;
        self.updated_at = Utc::now();
    }
    
    /// Updates when the article was published
    pub fn update_published_at(&mut self, published_at: DateTime<Utc>) {
        self.published_at = Some(published_at);
        self.updated_at = Utc::now();
    }
    
    /// Updates the read status of the article
    pub fn update_read_status(&mut self, read_status: ReadStatus) {
        self.read_status = read_status;
        self.updated_at = Utc::now();
    }
    
    /// Updates whether the article is favorited
    pub fn update_favorited(&mut self, is_favorited: bool) {
        self.is_favorited = is_favorited;
        self.updated_at = Utc::now();
    }
}

impl fmt::Display for Article {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_article_creation() {
        let feed_id = FeedId(Uuid::new_v4().to_string());
        let title = "Test Article".to_string();
        let url = Url::parse("https://example.com/article").unwrap();
        let article = Article::new(feed_id, title.clone(), url.clone());
        
        assert_eq!(article.feed_id, feed_id);
        assert_eq!(article.title, title);
        assert_eq!(article.url, url);
        assert!(article.category_id.is_none());
        assert!(article.author.is_none());
        assert!(article.content.is_none());
        assert!(article.summary.is_none());
        assert_eq!(article.read_status, ReadStatus::Unread);
        assert!(!article.is_favorited);
    }
    
    #[test]
    fn test_article_with_category() {
        let category_id = CategoryId(Uuid::new_v4().to_string());
        let article = Article::new(
            FeedId(Uuid::new_v4().to_string()),
            "Test Article".to_string(),
            Url::parse("https://example.com/article").unwrap(),
        )
        .with_category(category_id);
        
        assert_eq!(article.category_id, Some(category_id));
    }
    
    #[test]
    fn test_article_with_author() {
        let author = "Test Author".to_string();
        let article = Article::new(
            FeedId(Uuid::new_v4().to_string()),
            "Test Article".to_string(),
            Url::parse("https://example.com/article").unwrap(),
        )
        .with_author(author.clone());
        
        assert_eq!(article.author, Some(author));
    }
    
    #[test]
    fn test_article_with_content() {
        let content = "Test Content".to_string();
        let article = Article::new(
            FeedId(Uuid::new_v4().to_string()),
            "Test Article".to_string(),
            Url::parse("https://example.com/article").unwrap(),
        )
        .with_content(content.clone());
        
        assert_eq!(article.content, Some(content));
    }
    
    #[test]
    fn test_article_with_summary() {
        let summary = "Test Summary".to_string();
        let article = Article::new(
            FeedId(Uuid::new_v4().to_string()),
            "Test Article".to_string(),
            Url::parse("https://example.com/article").unwrap(),
        )
        .with_summary(summary.clone());
        
        assert_eq!(article.summary, Some(summary));
    }
    
    #[test]
    fn test_article_with_published_at() {
        let date = Utc::now();
        let article = Article::new(
            FeedId(Uuid::new_v4().to_string()),
            "Test Article".to_string(),
            Url::parse("https://example.com/article").unwrap(),
        )
        .with_published_at(date);
        
        assert_eq!(article.published_at, Some(date));
    }
    
    #[test]
    fn test_article_with_read_status() {
        let article = Article::new(
            FeedId(Uuid::new_v4().to_string()),
            "Test Article".to_string(),
            Url::parse("https://example.com/article").unwrap(),
        )
        .with_read_status(ReadStatus::Read);
        
        assert_eq!(article.read_status, ReadStatus::Read);
    }
    
    #[test]
    fn test_article_with_favorited() {
        let article = Article::new(
            FeedId(Uuid::new_v4().to_string()),
            "Test Article".to_string(),
            Url::parse("https://example.com/article").unwrap(),
        )
        .with_favorited(true);
        
        assert!(article.is_favorited);
    }
    
    #[test]
    fn test_article_update() {
        let mut article = Article::new(
            FeedId(Uuid::new_v4().to_string()),
            "Test Article".to_string(),
            Url::parse("https://example.com/article").unwrap(),
        );
        
        let category_id = CategoryId(Uuid::new_v4().to_string());
        article.update_category(Some(category_id));
        assert_eq!(article.category_id, Some(category_id));
        
        article.update_author(Some("Updated Author".to_string()));
        assert_eq!(article.author, Some("Updated Author".to_string()));
        
        article.update_content(Some("Updated Content".to_string()));
        assert_eq!(article.content, Some("Updated Content".to_string()));
        
        article.update_summary(Some("Updated Summary".to_string()));
        assert_eq!(article.summary, Some("Updated Summary".to_string()));
        
        let date = Utc::now();
        article.update_published_at(date);
        assert_eq!(article.published_at, Some(date));
        
        article.update_read_status(ReadStatus::Read);
        assert_eq!(article.read_status, ReadStatus::Read);
        
        article.update_favorited(true);
        assert!(article.is_favorited);
    }
}