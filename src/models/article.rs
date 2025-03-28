use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use url::Url;
use std::fmt;
use std::str::FromStr;
use std::fmt::Display;
use uuid;

use super::feed::{FeedId, Tag};

/// Unique identifier for articles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArticleId(pub String);

impl fmt::Display for ArticleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the read status of an article
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReadStatus {
    Unread,
    Read,
    Archived,
}

impl ReadStatus {
    pub fn to_string(&self) -> String {
        match self {
            ReadStatus::Unread => "unread".to_string(),
            ReadStatus::Read => "read".to_string(),
            ReadStatus::Archived => "archived".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "unread" => Some(ReadStatus::Unread),
            "read" => Some(ReadStatus::Read),
            "archived" => Some(ReadStatus::Archived),
            _ => None,
        }
    }
}

/// Represents an article from an RSS/Atom feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    /// Unique identifier for the article
    pub id: ArticleId,
    /// ID of the feed this article belongs to
    pub feed_id: String,
    /// Title of the article
    pub title: String,
    /// Original URL to the article
    pub url: Url,
    /// Author of the article
    pub author: Option<String>,
    /// Full content of the article (if available)
    pub content: String,
    /// Summary or excerpt of the article
    pub summary: Option<String>,
    /// Publication date of the article
    pub published_at: DateTime<Utc>,
    /// The last time the article was modified
    pub updated_at: DateTime<Utc>,
    /// Time when the article was saved
    pub created_at: DateTime<Utc>,
    /// Status of the article (read, unread, etc.)
    pub read_status: bool,
    /// Whether the article is favorited/starred
    pub is_favorite: bool,
    /// Thumbnail image URL
    pub thumbnail_url: Option<Url>,
    /// Tags applied to this article
    pub tags: Vec<String>,
}

impl Article {
    /// Creates a new article with the given title, URL, and feed ID
    pub fn new(feed_id: String, title: String, url: Url) -> Self {
        Self {
            id: ArticleId(uuid::Uuid::new_v4().to_string()),
            feed_id,
            title,
            url,
            author: None,
            content: String::new(),
            summary: None,
            published_at: Utc::now(),
            updated_at: Utc::now(),
            created_at: Utc::now(),
            read_status: false,
            is_favorite: false,
            thumbnail_url: None,
            tags: Vec::new(),
        }
    }
    
    /// Marks the article as read
    pub fn mark_as_read(&mut self) {
        self.read_status = true;
    }
    
    /// Marks the article as unread
    pub fn mark_as_unread(&mut self) {
        self.read_status = false;
    }
    
    /// Marks the article to read later
    pub fn mark_for_later(&mut self) {
        self.read_status = true;
    }
    
    /// Toggles the favorite status of the article
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
    }
    
    /// Adds a tag to the article
    pub fn add_tag(&mut self, tag: String) {
        self.tags.push(tag);
    }
    
    /// Removes a tag from the article
    pub fn remove_tag(&mut self, tag: &String) {
        self.tags.retain(|t| t != tag);
    }
    
    /// Returns true if the article has a specific tag
    pub fn has_tag(&self, tag_id: &str) -> bool {
        self.tags.iter().any(|t| t == tag_id)
    }
}