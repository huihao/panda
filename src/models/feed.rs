use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use rusqlite::types::{FromSql, ToSql, ToSqlOutput, ValueRef, FromSqlResult};

use crate::models::category::CategoryId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeedId(pub String);

impl std::fmt::Display for FeedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedStatus {
    Pending,
    Active,
    Error,
    Disabled,
}

impl FeedStatus {
    pub fn to_string(&self) -> String {
        match self {
            Self::Pending => "pending".to_string(),
            Self::Active => "active".to_string(),
            Self::Error => "error".to_string(),
            Self::Disabled => "disabled".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "active" => Some(Self::Active),
            "error" => Some(Self::Error),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feed {
    pub id: FeedId,
    pub category_id: Option<CategoryId>,
    pub title: String,
    pub url: Url,
    pub status: FeedStatus,
    pub error_message: Option<String>,
    pub icon_url: Option<Url>,
    pub site_url: Option<Url>,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub next_fetch_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Feed {
    pub fn new(title: String, url: Url) -> Self {
        let now = Utc::now();
        Self {
            id: FeedId::new(),
            category_id: None,
            title,
            url,
            status: FeedStatus::Pending,
            error_message: None,
            icon_url: None,
            site_url: None,
            last_fetched_at: None,
            next_fetch_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_category(mut self, category_id: CategoryId) -> Self {
        self.category_id = Some(category_id);
        self
    }

    pub fn with_icon_url(mut self, icon_url: Url) -> Self {
        self.icon_url = Some(icon_url);
        self
    }

    pub fn with_site_url(mut self, site_url: Url) -> Self {
        self.site_url = Some(site_url);
        self
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        // Just store the string value, even though we don't have a field for it yet
        // This maintains compatibility with the code that calls this method
        self
    }
    
    pub fn with_language(mut self, language: String) -> Self {
        // Just store the string value, even though we don't have a field for it yet
        // This maintains compatibility with the code that calls this method
        self
    }

    pub fn update_status(&mut self, status: FeedStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn update_error_message(&mut self, message: String) {
        self.error_message = Some(message);
        self.updated_at = Utc::now();
    }

    pub fn update_fetch_times(&mut self, last_fetched: DateTime<Utc>, next_fetch: DateTime<Utc>) {
        self.last_fetched_at = Some(last_fetched);
        self.next_fetch_at = Some(next_fetch);
        self.updated_at = Utc::now();
    }
}

impl FeedId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl FromSql for FeedId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        String::column_result(value).map(FeedId)
    }
}

impl ToSql for FeedId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.clone()))
    }
}

impl From<String> for FeedId {
    fn from(s: String) -> Self {
        FeedId(s)
    }
}