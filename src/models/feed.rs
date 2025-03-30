use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc};
use url::Url;
use std::collections::HashSet;
use uuid;
use uuid::Uuid;
use std::fmt;
use std::str::FromStr;
use anyhow::Result;
use std::sync::Arc;

use super::article::ReadStatus;
use super::category::{Category, CategoryId};

/// A unique identifier for a feed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeedId(pub String);

impl fmt::Display for FeedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FeedId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

/// Represents a tag that can be applied to articles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag {
    /// Unique identifier for the tag
    pub id: String,
    /// Display name of the tag
    pub name: String,
    /// Optional color for the tag (hex format, e.g., "#FF5733")
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Feed status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedStatus {
    Enabled,
    Disabled,
    Error,
}

/// Represents an RSS/Atom feed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feed {
    /// The unique identifier of the feed
    pub id: FeedId,
    
    /// The ID of the category this feed belongs to (if any)
    pub category_id: Option<CategoryId>,
    
    /// The title of the feed
    pub title: String,
    
    /// The description of the feed
    pub description: Option<String>,
    
    /// The URL of the feed
    pub url: Url,
    
    /// The URL of the feed's icon (if any)
    pub icon_url: Option<Url>,
    
    /// The URL of the feed's website (if any)
    pub site_url: Option<Url>,
    
    /// The language of the feed (if any)
    pub language: Option<String>,
    
    /// When the feed was last fetched
    pub last_fetched_at: Option<DateTime<Utc>>,
    
    /// The result of the last fetch (if any)
    pub last_fetch_error: Option<String>,
    
    /// When the feed should be fetched next
    pub next_fetch_at: Option<DateTime<Utc>>,
    
    /// The interval between fetches (in minutes)
    pub update_interval: i64,
    
    /// Whether the feed is enabled
    pub status: FeedStatus,
    
    /// When the feed was created in the database
    pub created_at: DateTime<Utc>,
    
    /// When the feed was last updated in the database
    pub updated_at: DateTime<Utc>,

    /// Additional error message for the feed
    pub error_message: Option<String>,
}

impl Feed {
    /// Creates a new feed with the given title and URL
    pub fn new(title: String, url: Url) -> Self {
        let now = Utc::now();
        Self {
            id: FeedId::new(),
            category_id: None,
            title,
            description: None,
            url,
            icon_url: None,
            site_url: None,
            language: None,
            last_fetched_at: None,
            last_fetch_error: None,
            next_fetch_at: None,
            update_interval: 3600,
            status: FeedStatus::default(),
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Sets the category ID of the feed
    pub fn with_category(mut self, category_id: CategoryId) -> Self {
        self.category_id = Some(category_id);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the description of the feed
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the icon URL of the feed
    pub fn with_icon_url(mut self, icon_url: Url) -> Self {
        self.icon_url = Some(icon_url);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the site URL of the feed
    pub fn with_site_url(mut self, site_url: Url) -> Self {
        self.site_url = Some(site_url);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the language of the feed
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the last fetched at time of the feed
    pub fn with_last_fetched_at(mut self, last_fetched_at: DateTime<Utc>) -> Self {
        self.last_fetched_at = Some(last_fetched_at);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the next fetch at time of the feed
    pub fn with_next_fetch_at(mut self, next_fetch_at: DateTime<Utc>) -> Self {
        self.next_fetch_at = Some(next_fetch_at);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the update interval of the feed
    pub fn with_update_interval(mut self, update_interval: i64) -> Self {
        self.update_interval = update_interval;
        self.updated_at = Utc::now();
        self
    }

    /// Updates the status of the feed
    pub fn update_status(&mut self, status: FeedStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Updates the error message of the feed
    pub fn update_error_message(&mut self, error_message: String) {
        self.error_message = Some(error_message);
        self.updated_at = Utc::now();
    }

    /// Updates the fetch times of the feed
    pub fn update_fetch_times(&mut self, last_fetched_at: DateTime<Utc>, next_fetch_at: DateTime<Utc>) {
        self.last_fetched_at = Some(last_fetched_at);
        self.next_fetch_at = Some(next_fetch_at);
        self.updated_at = Utc::now();
    }
}

impl fmt::Display for Feed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl FeedStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "enabled" => Self::Enabled,
            "disabled" => Self::Disabled,
            "error" => Self::Error,
            _ => Self::Enabled,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Enabled => "enabled".to_string(),
            Self::Disabled => "disabled".to_string(),
            Self::Error => "error".to_string(),
        }
    }
}

impl Default for FeedStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feed_creation() {
        let url = Url::parse("https://example.com/feed.xml").unwrap();
        let title = "Test Feed".to_string();
        let feed = Feed::new(title.clone(), url.clone());
        
        assert_eq!(feed.title, title);
        assert_eq!(feed.url, url);
        assert!(feed.description.is_none());
        assert!(feed.icon_url.is_none());
        assert!(feed.category_id.is_none());
        assert_eq!(feed.status, FeedStatus::Enabled);
        assert!(feed.last_fetched_at.is_none());
        assert!(feed.last_fetch_error.is_none());
        assert!(feed.next_fetch_at.is_none());
        assert_eq!(feed.update_interval, 3600);
    }
    
    #[test]
    fn test_feed_with_category() {
        let feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap())
            .with_category("test-category".to_string());
        
        assert_eq!(feed.category_id, Some("test-category".to_string()));
    }
    
    #[test]
    fn test_feed_with_description() {
        let description = "Test Description".to_string();
        let feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap())
            .with_description(description.clone());
        
        assert_eq!(feed.description, Some(description));
    }
    
    #[test]
    fn test_feed_with_icon_url() {
        let icon_url = Url::parse("https://example.com/icon.png").unwrap();
        let feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap())
            .with_icon_url(icon_url.clone());
        
        assert_eq!(feed.icon_url, Some(icon_url));
    }
    
    #[test]
    fn test_feed_with_site_url() {
        let site_url = Url::parse("https://example.com").unwrap();
        let feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap())
            .with_site_url(site_url.clone());
        
        assert_eq!(feed.site_url, Some(site_url));
    }
    
    #[test]
    fn test_feed_with_language() {
        let language = "en-US".to_string();
        let feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap())
            .with_language(language.clone());
        
        assert_eq!(feed.language, Some(language));
    }
    
    #[test]
    fn test_feed_with_last_fetched_at() {
        let date = Utc::now();
        let feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap())
            .with_last_fetched_at(date);
        
        assert_eq!(feed.last_fetched_at, Some(date));
    }
    
    #[test]
    fn test_feed_with_next_fetch() {
        let date = Utc::now();
        let feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap())
            .with_next_fetch_at(date);
        
        assert_eq!(feed.next_fetch_at, Some(date));
    }
    
    #[test]
    fn test_feed_with_update_interval() {
        let interval = 7200;
        let feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap())
            .with_update_interval(interval);
        
        assert_eq!(feed.update_interval, interval);
    }
    
    #[test]
    fn test_feed_update() {
        let mut feed = Feed::new("Test Feed".to_string(), Url::parse("https://example.com/feed.xml").unwrap());
        
        feed.update_title("Updated Feed".to_string());
        assert_eq!(feed.title, "Updated Feed");
        
        feed.update_category("new-category".to_string());
        assert_eq!(feed.category_id, Some("new-category".to_string()));
        
        feed.update_description("Updated Description".to_string());
        assert_eq!(feed.description, Some("Updated Description".to_string()));
        
        feed.update_icon_url(Url::parse("https://example.com/new-icon.png").unwrap());
        assert_eq!(feed.icon_url, Some(Url::parse("https://example.com/new-icon.png").unwrap()));
        
        feed.update_site_url(Url::parse("https://example.com/new-site").unwrap());
        assert_eq!(feed.site_url, Some(Url::parse("https://example.com/new-site").unwrap()));
        
        feed.update_language("fr-FR".to_string());
        assert_eq!(feed.language, Some("fr-FR".to_string()));
        
        let date = Utc::now();
        feed.update_fetch_times(date, date);
        assert_eq!(feed.last_fetched_at, Some(date));
        assert_eq!(feed.next_fetch_at, Some(date));
        
        feed.update_status(FeedStatus::Disabled);
        assert_eq!(feed.status, FeedStatus::Disabled);
        
        feed.update_error_message("Updated Error Message".to_string());
        assert_eq!(feed.error_message, Some("Updated Error Message".to_string()));
    }
}