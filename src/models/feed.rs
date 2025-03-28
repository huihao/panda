use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc};
use url::Url;
use std::collections::HashSet;
use uuid;
use uuid::Uuid;
use std::fmt;
use std::str::FromStr;

use super::article::ReadStatus;

/// Unique identifier for feeds
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeedId(pub String);

impl fmt::Display for FeedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

/// Unique identifier for categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryId(pub String);

/// Represents a category for organizing feeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier for the category
    pub id: CategoryId,
    /// Name of the category
    pub name: String,
    /// Optional description of the category
    pub description: Option<String>,
    /// Parent category (if any)
    pub parent_id: Option<CategoryId>,
    /// When this category was created
    pub created_at: DateTime<Utc>,
    /// When this category was last updated
    pub updated_at: DateTime<Utc>,
}

impl Category {
    /// Creates a new category with the given name
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: CategoryId(Uuid::new_v4().to_string()),
            name,
            description: None,
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Represents an RSS/Atom feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    /// Unique identifier for the feed
    pub id: FeedId,
    /// URL to the feed XML
    pub url: Url,
    /// Title of the feed
    pub title: String,
    /// Optional description of the feed
    pub description: Option<String>,
    /// URL to the feed's icon
    pub icon_url: Option<Url>,
    /// Category this feed belongs to (if any)
    pub category_id: Option<CategoryId>,
    /// Whether this feed is active
    pub status: FeedStatus,
    /// Error message for the feed
    pub error_message: Option<String>,
    /// When this feed was last fetched by the application
    pub last_fetch: Option<DateTime<Utc>>,
    /// Next fetch time
    pub next_fetch: Option<DateTime<Utc>>,
    /// Update interval in seconds
    pub update_interval: Option<i64>,
    /// When this feed was created
    pub created_at: DateTime<Utc>,
    /// When this feed was last updated
    pub updated_at: DateTime<Utc>,
}

impl Feed {
    /// Creates a new feed with the given title, URL, and description
    pub fn new(url: Url, title: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: FeedId(Uuid::new_v4().to_string()),
            url,
            title,
            description,
            icon_url: None,
            category_id: None,
            status: FeedStatus::Active,
            error_message: None,
            last_fetch: None,
            next_fetch: Some(now),
            update_interval: Some(300), // 5 minutes in seconds
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Determines if the feed should be updated based on the update interval
    pub fn should_update(&self) -> bool {
        if self.status != FeedStatus::Active {
            return false;
        }
        
        match self.last_fetch {
            Some(last_fetched) => {
                let now = Utc::now();
                let update_duration = Duration::seconds(self.update_interval.unwrap_or(0));
                now - last_fetched > update_duration
            }
            None => true, // Never fetched before, so should update
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeedStatus {
    Active,
    Error,
    Disabled,
}

impl FeedStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "error" => Self::Error,
            "disabled" => Self::Disabled,
            _ => Self::Active,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Active => "active".to_string(),
            Self::Error => "error".to_string(),
            Self::Disabled => "disabled".to_string(),
        }
    }
}

impl Default for FeedStatus {
    fn default() -> Self {
        Self::Active
    }
}