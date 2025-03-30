use std::str::FromStr;
use anyhow::Result;
use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row, OptionalExtension};
use url::Url;
use std::sync::Arc;
use rusqlite::{
    ToSql,
    types::{FromSql, ValueRef, FromSqlResult, FromSqlError},
};

// Fix the import path to use the external crate reference
use crate::base::repository::FeedRepository;
use crate::models::feed::{Feed, FeedId, FeedStatus, CategoryId};

// Wrapper types for SQLite serialization
#[derive(Debug)]
struct DateTimeWrapper(DateTime<Utc>);

impl FromSql for DateTimeWrapper {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let timestamp = i64::column_result(value)?;
        Ok(DateTimeWrapper(DateTime::from_timestamp(timestamp, 0).unwrap_or_default()))
    }
}

impl ToSql for DateTimeWrapper {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.0.timestamp().into())
    }
}

#[derive(Debug)]
struct UrlWrapper(Url);

impl FromSql for UrlWrapper {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let url_str = String::column_result(value)?;
        Url::parse(&url_str)
            .map(UrlWrapper)
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

impl ToSql for UrlWrapper {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.0.to_string().into())
    }
}

// Implement ToSql for CategoryId
impl ToSql for CategoryId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.0.to_string().into())
    }
}

pub struct SqliteFeedRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl SqliteFeedRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    fn map_row(row: &Row) -> Result<Feed> {
        Ok(Feed {
            id: FeedId(row.get(0)?),
            title: row.get(1)?,
            url: Url::parse(&row.get::<_, String>(2)?)?,
            description: row.get(3)?,
            category_id: row.get::<_, Option<String>>(4)?.map(CategoryId),
            icon_url: row.get(6).map(|url: String| Url::parse(&url).ok()).flatten(),
            site_url: row.get(7).map(|url: String| Url::parse(&url).ok()).flatten(),
            language: row.get(8)?,
            last_fetched_at: row.get(9).map(|date: String| DateTime::parse_from_rfc3339(&date).ok().map(|dt| dt.with_timezone(&Utc))).flatten(),
            next_fetch_at: row.get(10).map(|date: String| DateTime::parse_from_rfc3339(&date).ok().map(|dt| dt.with_timezone(&Utc))).flatten(),
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(12)?)?.with_timezone(&Utc),
            last_fetch_error: row.get(5)?,
            update_interval: row.get(13)?,
            status: FeedStatus::from_str(&row.get::<_, String>(14)?),
            error_message: row.get(15)?,
        })
    }
}

impl FeedRepository for SqliteFeedRepository {
    fn save_feed(&self, feed: &Feed) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO feeds (
                id, title, url, description, icon_url, site_url,
                category_id, language, last_fetched_at, last_fetch_error,
                next_fetch_at, update_interval, status, error_message,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        stmt.execute(params![
            feed.id.0,
            feed.title,
            feed.url.to_string(),
            feed.description,
            feed.icon_url.as_ref().map(|url| url.to_string()),
            feed.site_url.as_ref().map(|url| url.to_string()),
            feed.category_id.as_ref().map(|id| id.0.clone()),
            feed.language,
            feed.last_fetched_at.map(|dt| dt.to_rfc3339()),
            feed.last_fetch_error,
            feed.next_fetch_at.map(|dt| dt.to_rfc3339()),
            feed.update_interval,
            feed.status.to_string(),
            feed.error_message,
            feed.created_at.to_rfc3339(),
            feed.updated_at.to_rfc3339(),
        ])?;

        Ok(())
    }

    fn get_feed_by_id(&self, id: &FeedId) -> Result<Option<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, url, description, icon_url, site_url,
                    category_id, language, last_fetched_at, last_fetch_error,
                    next_fetch_at, update_interval, status, error_message,
                    created_at, updated_at
             FROM feeds WHERE id = ?"
        )?;

        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_row(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, url, description, icon_url, site_url,
                    category_id, language, last_fetched_at, last_fetch_error,
                    next_fetch_at, update_interval, status, error_message,
                    created_at, updated_at
             FROM feeds"
        )?;

        let mut feeds = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            feeds.push(Self::map_row(row)?);
        }

        Ok(feeds)
    }

    fn get_feeds_by_category(&self, category_id: &CategoryId) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT f.id, f.title, f.url, f.description, f.icon_url, f.site_url,
                    f.category_id, f.language, f.last_fetched_at, f.last_fetch_error,
                    f.next_fetch_at, f.update_interval, f.status, f.error_message,
                    f.created_at, f.updated_at
             FROM feeds f
             WHERE f.category_id = ?"
        )?;

        let mut feeds = Vec::new();
        let mut rows = stmt.query(params![category_id.0])?;
        while let Some(row) = rows.next()? {
            feeds.push(Self::map_row(row)?);
        }

        Ok(feeds)
    }

    fn get_feeds_that_need_update(&self) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, url, description, icon_url, site_url,
                    category_id, language, last_fetched_at, last_fetch_error,
                    next_fetch_at, update_interval, status, error_message,
                    created_at, updated_at
             FROM feeds
             WHERE next_fetch_at <= datetime('now')"
        )?;

        let mut feeds = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            feeds.push(Self::map_row(row)?);
        }

        Ok(feeds)
    }

    fn get_feeds_by_title(&self, title: &str) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, url, description, icon_url, site_url,
                    category_id, language, last_fetched_at, last_fetch_error,
                    next_fetch_at, update_interval, status, error_message,
                    created_at, updated_at
             FROM feeds
             WHERE title LIKE ?1
             ORDER BY title"
        )?;
        let rows = stmt.query_map(params![format!("%{}%", title)], Self::map_row)?;
        let mut feeds = Vec::new();
        for feed in rows {
            feeds.push(feed?);
        }
        Ok(feeds)
    }

    fn update_feed(&self, feed: &Feed) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE feeds
             SET title = ?1, url = ?2, description = ?3, icon_url = ?4,
                 site_url = ?5, category_id = ?6, language = ?7,
                 last_fetched_at = ?8, last_fetch_error = ?9,
                 next_fetch_at = ?10, update_interval = ?11,
                 status = ?12, error_message = ?13, updated_at = ?14
             WHERE id = ?15",
            params![
                feed.title,
                feed.url.to_string(),
                feed.description,
                feed.icon_url.as_ref().map(|url| url.to_string()),
                feed.site_url.as_ref().map(|url| url.to_string()),
                feed.category_id.as_ref().map(|id| id.0.clone()),
                feed.language,
                feed.last_fetched_at.map(|dt| dt.to_rfc3339()),
                feed.last_fetch_error,
                feed.next_fetch_at.map(|dt| dt.to_rfc3339()),
                feed.update_interval,
                feed.status.to_string(),
                feed.error_message,
                feed.updated_at.to_rfc3339(),
                feed.id.0,
            ],
        )?;
        Ok(())
    }

    fn delete_feed(&self, id: &FeedId) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM feeds WHERE id = ?1", params![id.0])?;
        Ok(())
    }

    fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, url, description, icon_url, site_url,
                    category_id, language, last_fetched_at, last_fetch_error,
                    next_fetch_at, update_interval, status, error_message,
                    created_at, updated_at
             FROM feeds
             WHERE url = ?"
        )?;
        let mut rows = stmt.query(params![url])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_row(row)?))
        } else {
            Ok(None)
        }
    }

    fn search_feeds(&self, query: &str) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, url, description, icon_url, site_url,
                    category_id, language, last_fetched_at, last_fetch_error,
                    next_fetch_at, update_interval, status, error_message,
                    created_at, updated_at
             FROM feeds
             WHERE title LIKE ? OR description LIKE ?"
        )?;
        let pattern = format!("%{}%", query);
        let mut rows = stmt.query(params![pattern, pattern])?;
        let mut feeds = Vec::new();
        while let Some(row) = rows.next()? {
            feeds.push(Self::map_row(row)?);
        }
        Ok(feeds)
    }

    fn get_feeds_to_update(&self) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM feeds WHERE next_fetch_at <= ?")?;
        let now = Utc::now().to_rfc3339();
        let mut rows = stmt.query(params![now])?;
        let mut feeds = Vec::new();
        while let Some(row) = rows.next()? {
            feeds.push(Self::map_row(row)?);
        }
        Ok(feeds)
    }

    fn get_feeds_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM feeds WHERE last_fetched_at BETWEEN ? AND ?")?;
        let mut rows = stmt.query(params![start.to_rfc3339(), end.to_rfc3339()])?;
        let mut feeds = Vec::new();
        while let Some(row) = rows.next()? {
            feeds.push(Self::map_row(row)?);
        }
        Ok(feeds)
    }
}