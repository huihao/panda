use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use chrono::{DateTime, Utc};
use anyhow::Result;
use async_trait::async_trait;

use crate::models::feed::{Feed, FeedId, FeedStatus};
use crate::models::category::CategoryId;
use crate::base::repository_traits::FeedRepository;

pub struct SqliteFeedRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteFeedRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    fn map_row(&self, row: &rusqlite::Row) -> Result<Feed> {
        Ok(Feed {
            id: row.get::<_, String>(0)?.into(),
            category_id: row.get::<_, Option<String>>(1)?.map(|s| s.into()),
            title: row.get(2)?,
            url: row.get(3)?,
            status: FeedStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(FeedStatus::Pending),
            error_message: row.get(5)?,
            icon_url: row.get(6)?,
            site_url: row.get(7)?,
            last_fetched_at: row.get(8)?,
            next_fetch_at: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

#[async_trait]
impl FeedRepository for SqliteFeedRepository {
    async fn get_feed_by_id(&self, id: &FeedId) -> Result<Option<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             WHERE id = ?"
        )?;

        let mut rows = stmt.query([id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.map_row(row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             WHERE url = ?"
        )?;

        let mut rows = stmt.query([url])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.map_row(row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             ORDER BY title"
        )?;

        let rows = stmt.query_map([], |row| Ok(self.map_row(row)))?;
        let feeds = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    async fn get_feeds_by_category(&self, category_id: &CategoryId) -> Result<Vec<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             WHERE category_id = ? 
             ORDER BY title"
        )?;

        let rows = stmt.query_map([category_id.to_string()], |row| Ok(self.map_row(row)))?;
        let feeds = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    async fn get_enabled_feeds(&self) -> Result<Vec<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             WHERE status = ? 
             ORDER BY title"
        )?;

        let rows = stmt.query_map([FeedStatus::Active.to_string()], |row| Ok(self.map_row(row)))?;
        let feeds = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    async fn get_feeds_to_update(&self) -> Result<Vec<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             WHERE next_fetch_at <= ? OR next_fetch_at IS NULL"
        )?;

        let now = Utc::now();
        let rows = stmt.query_map([now], |row| Ok(self.map_row(row)))?;
        let feeds = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    async fn search_feeds(&self, query: &str) -> Result<Vec<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let search_term = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             WHERE title LIKE ? OR url LIKE ? 
             ORDER BY title"
        )?;

        let rows = stmt.query_map([&search_term, &search_term], |row| Ok(self.map_row(row)))?;
        let feeds = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    async fn get_feeds_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             WHERE created_at BETWEEN ? AND ? 
             ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map([start, end], |row| Ok(self.map_row(row)))?;
        let feeds = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    async fn get_recently_updated_feeds(&self, limit: usize) -> Result<Vec<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, category_id, title, url, status, error_message, icon_url, site_url,
                    last_fetched_at, next_fetch_at, created_at, updated_at 
             FROM feeds 
             ORDER BY updated_at DESC 
             LIMIT ?"
        )?;

        let rows = stmt.query_map([limit as i64], |row| Ok(self.map_row(row)))?;
        let feeds = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    async fn get_most_active_feeds(&self, limit: usize) -> Result<Vec<Feed>> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT f.id, f.category_id, f.title, f.url, f.status, f.error_message, f.icon_url, f.site_url,
                    f.last_fetched_at, f.next_fetch_at, f.created_at, f.updated_at 
             FROM feeds f
             LEFT JOIN (
                SELECT feed_id, COUNT(*) as article_count
                FROM articles
                GROUP BY feed_id
             ) a ON f.id = a.feed_id
             ORDER BY COALESCE(a.article_count, 0) DESC 
             LIMIT ?"
        )?;

        let rows = stmt.query_map([limit as i64], |row| Ok(self.map_row(row)))?;
        let feeds = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    async fn save_feed(&self, feed: &Feed) -> Result<()> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO feeds (
                id, category_id, title, url, status, error_message, icon_url, site_url,
                last_fetched_at, next_fetch_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                feed.id.to_string(),
                feed.category_id.as_ref().map(|id| id.to_string()),
                feed.title,
                feed.url.to_string(),
                feed.status.to_string(),
                feed.error_message,
                feed.icon_url.as_ref().map(|u| u.to_string()),
                feed.site_url.as_ref().map(|u| u.to_string()),
                feed.last_fetched_at,
                feed.next_fetch_at,
                feed.created_at,
                feed.updated_at,
            ],
        )?;
        Ok(())
    }

    async fn update_feed(&self, feed: &Feed) -> Result<()> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "UPDATE feeds SET 
                category_id = ?,
                title = ?,
                url = ?,
                status = ?,
                error_message = ?,
                icon_url = ?,
                site_url = ?,
                last_fetched_at = ?,
                next_fetch_at = ?,
                updated_at = ?
            WHERE id = ?",
            rusqlite::params![
                feed.category_id.as_ref().map(|id| id.to_string()),
                feed.title,
                feed.url.to_string(),
                feed.status.to_string(),
                feed.error_message,
                feed.icon_url.as_ref().map(|u| u.to_string()),
                feed.site_url.as_ref().map(|u| u.to_string()),
                feed.last_fetched_at,
                feed.next_fetch_at,
                feed.updated_at,
                feed.id.to_string(),
            ],
        )?;
        Ok(())
    }

    async fn delete_feed(&self, id: &FeedId) -> Result<()> {
        // 锁定连接
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM feeds WHERE id = ?", [id.to_string()])?;
        Ok(())
    }
}