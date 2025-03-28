use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{error, info};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ValueRef};
use rusqlite::{params, Connection, Error as RusqliteError, OptionalExtension, Row};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use url::Url;
use uuid::Uuid;

// Fix the import path to use the external crate reference

use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::models::feed::{Feed, FeedId, Tag};
use crate::base::repository::ArticleRepository;

// Wrapper types for SQLite serialization
#[derive(Debug)]
struct DateTimeWrapper(DateTime<Utc>);

impl FromSql for DateTimeWrapper {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let timestamp = i64::column_result(value)?;
        Ok(DateTimeWrapper(
            DateTime::from_timestamp(timestamp, 0).unwrap_or_default(),
        ))
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

/// SQLite-based article repository implementation
pub struct SqliteArticleRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteArticleRepository {
    /// Creates a new SQLite article repository
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// Maps a database row to an Article
    fn map_row(&self, row: &Row) -> Result<Article, rusqlite::Error> {
        let id: String = row.get(0)?;
        let feed_id: String = row.get(1)?;
        let title: String = row.get(2)?;
        let url: String = row.get(3)?;
        let author: Option<String> = row.get(4)?;
        let content: String = row.get(5)?;
        let summary: Option<String> = row.get(6)?;
        let published_at: i64 = row.get(7)?;
        let read_status: bool = row.get(8)?;
        let is_favorite: bool = row.get(9)?;
        let created_at: i64 = row.get(10)?;
        let updated_at: i64 = row.get(11)?;
        let thumbnail_url: Option<String> = row.get(12)?;

        Ok(Article {
            id: ArticleId(id),
            feed_id,
            title,
            url: url.parse().unwrap(),
            author,
            content,
            summary,
            published_at: DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp(published_at, 0),
                Utc,
            ),
            read_status,
            is_favorite,
            created_at: DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp(created_at, 0),
                Utc,
            ),
            updated_at: DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp(updated_at, 0),
                Utc,
            ),
            tags: Default::default(),
            thumbnail_url: thumbnail_url.map(|u| u.parse().unwrap()),
        })
    }

    fn get_tags(&self, article_id: &ArticleId) -> Result<HashSet<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at
             FROM tags t
             JOIN article_tags at ON t.id = at.tag_id
             WHERE at.article_id = ?",
        )?;

        let tags = stmt.query_map([&article_id.0], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        Ok(tags.collect::<rusqlite::Result<HashSet<_>>>()?)
    }

    fn update_article_tags(&self, article_id: &ArticleId, tags: &HashSet<Tag>) -> Result<()> {
        let conn = self.pool.get()?;
        // First, delete all existing tags for this article
        let mut stmt = conn.prepare("DELETE FROM article_tags WHERE article_id = ?")?;
        stmt.execute([&article_id.0])?;

        // Then insert the new tags
        let mut stmt =
            conn.prepare("INSERT INTO article_tags (article_id, tag_id) VALUES (?, ?)")?;
        for tag in tags {
            stmt.execute(params![article_id.0, tag.id])?;
        }

        Ok(())
    }
}

impl ArticleRepository for SqliteArticleRepository {
    fn create_article(&self, article: &Article) -> Result<ArticleId> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT INTO articles (id, feed_id, title, url, author, content, summary, published_at, read_status, is_favorite, created_at, updated_at, thumbnail_url) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        stmt.execute(params![
            article.id.0,
            article.feed_id,
            article.title,
            article.url.to_string(),
            article.author,
            article.content,
            article.summary,
            article.published_at.timestamp(),
            article.read_status,
            article.is_favorite,
            article.created_at.timestamp(),
            article.updated_at.timestamp(),
            article.thumbnail_url.as_ref().map(|u| u.to_string()),
        ])?;

        Ok(article.id.clone())
    }

    fn get_article(&self, id: &ArticleId) -> Result<Option<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM articles WHERE id = ?")?;
        let article = stmt
            .query_row([&id.0], |row| self.map_row(row))
            .optional()?;
        Ok(article)
    }

    fn get_article_by_url(&self, url: &str) -> Result<Option<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM articles WHERE url = ?")?;
        let article = stmt.query_row([url], |row| self.map_row(row)).optional()?;
        Ok(article)
    }

    fn get_articles_by_feed(&self, feed_id: &str) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT * FROM articles WHERE feed_id = ? ORDER BY published_at DESC")?;
        let articles = stmt
            .query_map([feed_id], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    fn get_all_articles(&self) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM articles ORDER BY published_at DESC")?;
        let articles = stmt
            .query_map([], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    fn get_unread_articles(&self) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn
            .prepare("SELECT * FROM articles WHERE read_status = 0 ORDER BY published_at DESC")?;
        let articles = stmt
            .query_map([], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    fn get_articles_by_date(&self, date: DateTime<Utc>) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM articles WHERE DATE(published_at) = DATE(?) ORDER BY published_at DESC",
        )?;

        let articles = stmt
            .query_map(params![DateTimeWrapper(date)], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    fn update_article(&self, article: &Article) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "UPDATE articles SET 
             feed_id = ?, title = ?, url = ?, author = ?, content = ?, summary = ?, 
             published_at = ?, read_status = ?, is_favorite = ?, updated_at = ?, thumbnail_url = ?
             WHERE id = ?",
        )?;

        stmt.execute(params![
            article.feed_id,
            article.title,
            article.url.to_string(),
            article.author,
            article.content,
            article.summary,
            article.published_at.timestamp(),
            article.read_status,
            article.is_favorite,
            article.updated_at.timestamp(),
            article.thumbnail_url.as_ref().map(|u| u.to_string()),
            article.id.0,
        ])?;

        Ok(())
    }

    fn delete_article(&self, id: &ArticleId) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM articles WHERE id = ?")?;
        stmt.execute([&id.0])?;
        Ok(())
    }

    fn add_tags(&self, article_id: ArticleId, tags: &[String]) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO article_tags (article_id, tag) VALUES (?, ?)")?;

        for tag in tags {
            stmt.execute(params![article_id.0, tag])?;
        }

        Ok(())
    }

    fn get_article_tags(&self, article_id: ArticleId) -> Result<Vec<String>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT tag FROM article_tags WHERE article_id = ?")?;

        let tags = stmt
            .query_map(params![article_id.0], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    fn add_tag(&self, article_id: ArticleId, tag_id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO article_tags (article_id, tag) VALUES (?, ?)")?;

        stmt.execute(params![article_id.0, tag_id])?;
        Ok(())
    }

    fn remove_tag(&self, article_id: ArticleId, tag_id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM article_tags WHERE article_id = ? AND tag = ?")?;

        stmt.execute(params![article_id.0, tag_id])?;
        Ok(())
    }

    fn get_favorite_articles(&self) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn
            .prepare("SELECT * FROM articles WHERE is_favorite = 1 ORDER BY published_at DESC")?;
        let articles = stmt
            .query_map([], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    fn search_articles(&self, query: &str) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT * FROM articles 
             WHERE title LIKE ? 
             OR content LIKE ? 
             OR summary LIKE ? 
             OR author LIKE ?
             ORDER BY published_at DESC",
        )?;

        let articles = stmt
            .query_map(
                params![
                    &search_pattern,
                    &search_pattern,
                    &search_pattern,
                    &search_pattern
                ],
                |row| self.map_row(row),
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    fn cleanup_old_articles(&self, retention_days: i64) -> Result<usize> {
        let conn = self.pool.get()?;
        let cutoff_timestamp = Utc::now()
            .checked_sub_signed(chrono::Duration::days(retention_days))
            .ok_or_else(|| anyhow::anyhow!("Invalid retention period"))?
            .timestamp();

        let mut stmt = conn.prepare(
            "DELETE FROM articles 
             WHERE published_at < ? 
             AND is_favorite = 0",
        )?;

        let affected_rows = stmt.execute([cutoff_timestamp])?;
        Ok(affected_rows)
    }
}

// Implement ToSql for ArticleId
impl rusqlite::ToSql for ArticleId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(self.to_string()))
    }
}
