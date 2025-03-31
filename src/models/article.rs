use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use rusqlite::types::{FromSql, ToSql, ToSqlOutput, ValueRef, FromSqlResult};

use crate::models::feed::FeedId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArticleId(pub String);

impl std::fmt::Display for ArticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReadStatus {
    Unread,
    Read,
    Archived,
}

impl ReadStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "unread" => Some(Self::Unread),
            "read" => Some(Self::Read),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Unread => "unread".to_string(),
            Self::Read => "read".to_string(),
            Self::Archived => "archived".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: ArticleId,
    pub feed_id: FeedId,
    pub title: String,
    pub url: Url,
    pub author: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub published_at: DateTime<Utc>,
    pub read_status: ReadStatus,
    pub is_favorited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Article {
    pub fn new(feed_id: FeedId, title: String, url: Url) -> Self {
        let now = Utc::now();
        Self {
            id: ArticleId::new(),
            feed_id,
            title,
            url,
            author: None,
            content: None,
            summary: None,
            published_at: now,
            read_status: ReadStatus::Unread,
            is_favorited: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }

    pub fn with_published_at(mut self, published_at: DateTime<Utc>) -> Self {
        self.published_at = published_at;
        self
    }

    pub fn mark_as_read(&mut self) {
        self.read_status = ReadStatus::Read;
        self.updated_at = Utc::now();
    }

    pub fn mark_as_unread(&mut self) {
        self.read_status = ReadStatus::Unread;
        self.updated_at = Utc::now();
    }

    pub fn archive(&mut self) {
        self.read_status = ReadStatus::Archived;
        self.updated_at = Utc::now();
    }

    pub fn toggle_favorite(&mut self) {
        self.is_favorited = !self.is_favorited;
        self.updated_at = Utc::now();
    }
}

impl ArticleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl FromSql for ArticleId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        String::column_result(value).map(ArticleId)
    }
}

impl ToSql for ArticleId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.clone()))
    }
}

impl From<String> for ArticleId {
    fn from(s: String) -> Self {
        ArticleId(s)
    }
}