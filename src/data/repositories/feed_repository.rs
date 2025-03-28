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
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteFeedRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    fn map_row(&self, row: &Row) -> Result<Feed, rusqlite::Error> {
        let id: String = row.get(0)?;
        let url: String = row.get(1)?;
        let title: String = row.get(2)?;
        let description: Option<String> = row.get(3)?;
        let icon_url: Option<String> = row.get(4)?;
        let category_id: Option<String> = row.get(5)?;
        let status: String = row.get(6)?;
        let error_message: Option<String> = row.get(7)?;
        let last_fetch: Option<i64> = row.get(8)?;
        let next_fetch: Option<i64> = row.get(9)?;
        let update_interval: Option<i64> = row.get(10)?;
        let created_at: i64 = row.get(11)?;
        let updated_at: i64 = row.get(12)?;

        Ok(Feed {
            id: FeedId(id),
            url: url.parse().unwrap(),
            title,
            description,
            icon_url: icon_url.map(|u| u.parse().unwrap()),
            category_id: category_id.map(CategoryId),
            status: FeedStatus::from_str(&status),
            error_message,
            last_fetch: last_fetch.map(|ts| DateTime::from_utc(chrono::NaiveDateTime::from_timestamp(ts, 0), Utc)),
            next_fetch: next_fetch.map(|ts| DateTime::from_utc(chrono::NaiveDateTime::from_timestamp(ts, 0), Utc)),
            update_interval,
            created_at: DateTime::from_utc(chrono::NaiveDateTime::from_timestamp(created_at, 0), Utc),
            updated_at: DateTime::from_utc(chrono::NaiveDateTime::from_timestamp(updated_at, 0), Utc),
        })
    }
}

impl FeedRepository for SqliteFeedRepository {
    fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM feeds ORDER BY title")?;
        let feeds = stmt.query_map([], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }
    
    fn get_feed_by_id(&self, id: &FeedId) -> Result<Option<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM feeds WHERE id = ?")?;
        let feed = stmt.query_row([&id.0], |row| self.map_row(row)).optional()?;
        Ok(feed)
    }
    
    fn get_feeds_by_category(&self, category_id: &CategoryId) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM feeds WHERE category_id = ? ORDER BY title")?;
        let feeds = stmt.query_map([&category_id.0], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }
    
    fn save_feed(&self, feed: &Feed) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT INTO feeds (
                id, url, title, description, icon_url, category_id,
                status, error_message, last_fetch, next_fetch,
                update_interval, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        stmt.execute(params![
            feed.id.0,
            feed.url.to_string(),
            feed.title,
            feed.description,
            feed.icon_url.as_ref().map(|u| u.to_string()),
            feed.category_id,
            feed.status.to_string(),
            feed.error_message,
            feed.last_fetch.map(|dt| dt.timestamp()),
            feed.next_fetch.map(|dt| dt.timestamp()),
            feed.update_interval,
            feed.created_at.timestamp(),
            feed.updated_at.timestamp(),
        ])?;

        Ok(())
    }
    
    fn update_feed(&self, feed: &Feed) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "UPDATE feeds SET
                url = ?,
                title = ?,
                description = ?,
                icon_url = ?,
                category_id = ?,
                status = ?,
                error_message = ?,
                last_fetch = ?,
                next_fetch = ?,
                update_interval = ?,
                updated_at = ?
            WHERE id = ?"
        )?;

        stmt.execute(params![
            feed.url.to_string(),
            feed.title,
            feed.description,
            feed.icon_url.as_ref().map(|u| u.to_string()),
            feed.category_id,
            feed.status.to_string(),
            feed.error_message,
            feed.last_fetch.map(|dt| dt.timestamp()),
            feed.next_fetch.map(|dt| dt.timestamp()),
            feed.update_interval,
            feed.updated_at.timestamp(),
            feed.id.0,
        ])?;

        Ok(())
    }
    
    fn delete_feed(&self, id: &FeedId) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM feeds WHERE id = ?")?;
        stmt.execute([&id.0])?;
        Ok(())
    }

    fn search_feeds(&self, query: &str) -> Result<Vec<Feed>> {
        let conn = self.pool.get()?;
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT * FROM feeds 
             WHERE title LIKE ? 
             OR url LIKE ? 
             ORDER BY title"
        )?;
        
        let feeds = stmt.query_map(
            params![&search_pattern, &search_pattern],
            |row| self.map_row(row)
        )?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(feeds)
    }
    
    fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM feeds WHERE url = ?")?;
        let feed = stmt.query_row([url], |row| self.map_row(row)).optional()?;
        Ok(feed)
    }
}