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

use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::models::feed::{FeedId, CategoryId};
use crate::models::Tag;
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
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl SqliteArticleRepository {
    /// Creates a new SQLite article repository
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// Maps a database row to an Article
    fn map_row(row: &Row) -> Result<Article> {
        let id: String = row.get(0)?;
        let feed_id: String = row.get(1)?;
        let category_id: Option<String> = row.get(2)?;
        let title: String = row.get(3)?;
        let url: String = row.get(4)?;
        let author: Option<String> = row.get(5)?;
        let content: Option<String> = row.get(6)?;
        let summary: Option<String> = row.get(7)?;
        let published_at: Option<String> = row.get(8)?;
        let read_status: String = row.get(9)?;
        let is_favorited: bool = row.get(10)?;
        let created_at: String = row.get(11)?;
        let updated_at: String = row.get(12)?;

        let published_at = published_at
            .map(|dt| DateTime::parse_from_rfc3339(&dt))
            .transpose()?
            .map(|dt| dt.with_timezone(&Utc));

        let read_status = match read_status.as_str() {
            "unread" => ReadStatus::Unread,
            "read" => ReadStatus::Read,
            "archived" => ReadStatus::Archived,
            _ => ReadStatus::Unread,
        };

        let created_at = DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc);

        Ok(Article {
            id: ArticleId(id),
            feed_id: FeedId(feed_id),
            category_id: category_id.map(CategoryId),
            title,
            url: Url::parse(&url)?,
            author,
            content,
            summary,
            published_at,
            read_status,
            is_favorited,
            created_at,
            updated_at,
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
    fn save_article(&self, article: &Article) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO articles (id, feed_id, category_id, title, url, author, content, summary, published_at, read_status, is_favorited, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        stmt.execute(params![
            article.id.0,
            article.feed_id.0,
            article.category_id.as_ref().map(|id| id.0.clone()),
            article.title,
            article.url.to_string(),
            article.author,
            article.content,
            article.summary,
            article.published_at.map(|dt| dt.to_rfc3339()),
            article.read_status.to_string(),
            article.is_favorited,
            article.created_at.to_rfc3339(),
            article.updated_at.to_rfc3339(),
        ])?;

        Ok(())
    }

    fn get_article_by_id(&self, id: &ArticleId) -> Result<Option<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary, published_at, read_status, is_favorited, created_at, updated_at FROM articles WHERE id = ?"
        )?;

        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_row(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_articles_by_feed(&self, feed_id: &FeedId) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary, published_at, read_status, is_favorited, created_at, updated_at FROM articles WHERE feed_id = ?"
        )?;

        let mut articles = Vec::new();
        let mut rows = stmt.query(params![feed_id.0])?;
        while let Some(row) = rows.next()? {
            articles.push(Self::map_row(row)?);
        }

        Ok(articles)
    }

    fn get_favorited_articles(&self) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary, published_at, read_status, is_favorited, created_at, updated_at FROM articles WHERE is_favorited = 1"
        )?;

        let mut articles = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            articles.push(Self::map_row(row)?);
        }

        Ok(articles)
    }

    fn get_unread_articles(&self) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary, published_at, read_status, is_favorited, created_at, updated_at FROM articles WHERE read_status = 'unread'"
        )?;

        let mut articles = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            articles.push(Self::map_row(row)?);
        }

        Ok(articles)
    }

    fn get_articles_by_date(&self, date: DateTime<Utc>) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM articles WHERE DATE(published_at) = DATE(?) ORDER BY published_at DESC",
        )?;

        let articles = stmt
            .query_map(params![DateTimeWrapper(date)], |row| Self::map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    fn update_article(&self, article: &Article) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "UPDATE articles SET 
             feed_id = ?, title = ?, url = ?, author = ?, content = ?, summary = ?, 
             published_at = ?, read_status = ?, is_favorite = ?, saved_at = ?, category_id = ?
             WHERE id = ?",
        )?;

        stmt.execute(params![
            article.feed_id.0,
            article.title,
            article.url.to_string(),
            article.author,
            article.content,
            article.summary,
            article.published_at.map(|dt| dt.to_rfc3339()),
            article.read_status.to_string(),
            article.is_favorited,
            article.updated_at.to_rfc3339(),
            article.category_id.as_ref().map(|id| id.0.clone()),
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
        let mut stmt = conn.prepare("INSERT INTO article_tags (article_id, tag_id) VALUES (?, ?)")?;
        for tag in tags {
            stmt.execute(params![article_id.0, tag])?;
        }
        Ok(())
    }

    fn get_article_tags(&self, article_id: ArticleId) -> Result<Vec<String>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t
             JOIN article_tags at ON t.id = at.tag_id
             WHERE at.article_id = ?",
        )?;
        let tags = stmt
            .query_map([article_id.0], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    fn add_tag(&self, article_id: ArticleId, tag_id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO article_tags (article_id, tag_id) VALUES (?, ?)")?;
        stmt.execute(params![article_id.0, tag_id])?;
        Ok(())
    }

    fn remove_tag(&self, article_id: ArticleId, tag_id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM article_tags WHERE article_id = ? AND tag_id = ?")?;
        stmt.execute(params![article_id.0, tag_id])?;
        Ok(())
    }

    fn search_articles(&self, query: &str) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM articles 
             WHERE title LIKE ? OR content LIKE ? OR summary LIKE ?
             ORDER BY published_at DESC",
        )?;

        let pattern = format!("%{}%", query);
        let articles = stmt
            .query_map(params![pattern, pattern, pattern], |row| Self::map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    fn cleanup_old_articles(&self, retention_days: i64) -> Result<usize> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "DELETE FROM articles 
             WHERE published_at < datetime('now', ?) 
             AND read_status != 'archived'
             AND is_favorited = 0",
        )?;

        let days_pattern = format!("-{} days", retention_days);
        let count = stmt.execute([days_pattern])?;
        Ok(count)
    }

    /// Creates a new article
    pub fn create_article(&self, article: &Article) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT INTO articles (id, feed_id, title, url, author, content, summary, published_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        stmt.execute([
            &article.id.to_string(),
            &article.feed_id.to_string(),
            &article.title,
            &article.url,
            &article.author.as_deref().unwrap_or(""),
            &article.content.as_deref().unwrap_or(""),
            &article.summary.as_deref().unwrap_or(""),
            &article.published_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            &Utc::now().to_rfc3339(),
            &Utc::now().to_rfc3339(),
        ])?;

        Ok(())
    }

    /// Gets all articles
    pub fn get_all_articles(&self) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary, published_at, created_at, updated_at
             FROM articles
             ORDER BY published_at DESC"
        )?;

        let articles = stmt.query_map([], |row| {
            Ok(Article {
                id: ArticleId::from_str(row.get::<_, String>(0)?).unwrap(),
                feed_id: FeedId::from_str(row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                url: row.get(3)?,
                author: Some(row.get(4)?),
                content: Some(row.get(5)?),
                summary: Some(row.get(6)?),
                published_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.with_timezone(&Utc))
                    .ok(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Gets an article by ID
    pub fn get_article(&self, id: &ArticleId) -> Result<Option<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary, published_at, created_at, updated_at
             FROM articles
             WHERE id = ?"
        )?;

        let article = stmt.query_row([id.to_string()], |row| {
            Ok(Article {
                id: ArticleId::from_str(row.get::<_, String>(0)?).unwrap(),
                feed_id: FeedId::from_str(row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                url: row.get(3)?,
                author: Some(row.get(4)?),
                content: Some(row.get(5)?),
                summary: Some(row.get(6)?),
                published_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.with_timezone(&Utc))
                    .ok(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
            })
        })
        .optional()?;

        Ok(article)
    }

    /// Gets an article by URL
    pub fn get_article_by_url(&self, url: &str) -> Result<Option<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary, published_at, created_at, updated_at
             FROM articles
             WHERE url = ?"
        )?;

        let article = stmt.query_row([url], |row| {
            Ok(Article {
                id: ArticleId::from_str(row.get::<_, String>(0)?).unwrap(),
                feed_id: FeedId::from_str(row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                url: row.get(3)?,
                author: Some(row.get(4)?),
                content: Some(row.get(5)?),
                summary: Some(row.get(6)?),
                published_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.with_timezone(&Utc))
                    .ok(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
            })
        })
        .optional()?;

        Ok(article)
    }

    /// Gets articles by category
    pub fn get_articles_by_category(&self, category_id: &CategoryId) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT a.id, a.feed_id, a.title, a.url, a.author, a.content, a.summary, a.published_at, a.created_at, a.updated_at
             FROM articles a
             JOIN feeds f ON a.feed_id = f.id
             WHERE f.category_id = ?
             ORDER BY a.published_at DESC"
        )?;

        let articles = stmt.query_map([category_id.to_string()], |row| {
            Ok(Article {
                id: ArticleId::from_str(row.get::<_, String>(0)?).unwrap(),
                feed_id: FeedId::from_str(row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                url: row.get(3)?,
                author: Some(row.get(4)?),
                content: Some(row.get(5)?),
                summary: Some(row.get(6)?),
                published_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.with_timezone(&Utc))
                    .ok(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Gets favorite articles
    pub fn get_favorite_articles(&self) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT a.id, a.feed_id, a.title, a.url, a.author, a.content, a.summary, a.published_at, a.created_at, a.updated_at
             FROM articles a
             JOIN article_favorites af ON a.id = af.article_id
             ORDER BY a.published_at DESC"
        )?;

        let articles = stmt.query_map([], |row| {
            Ok(Article {
                id: ArticleId::from_str(row.get::<_, String>(0)?).unwrap(),
                feed_id: FeedId::from_str(row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                url: row.get(3)?,
                author: Some(row.get(4)?),
                content: Some(row.get(5)?),
                summary: Some(row.get(6)?),
                published_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.with_timezone(&Utc))
                    .ok(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Gets articles by date range
    pub fn get_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary, published_at, created_at, updated_at
             FROM articles
             WHERE published_at BETWEEN ? AND ?
             ORDER BY published_at DESC"
        )?;

        let articles = stmt.query_map(
            [start.to_rfc3339(), end.to_rfc3339()],
            |row| {
                Ok(Article {
                    id: ArticleId::from_str(row.get::<_, String>(0)?).unwrap(),
                    feed_id: FeedId::from_str(row.get::<_, String>(1)?).unwrap(),
                    title: row.get(2)?,
                    url: row.get(3)?,
                    author: Some(row.get(4)?),
                    content: Some(row.get(5)?),
                    summary: Some(row.get(6)?),
                    published_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|d| d.with_timezone(&Utc))
                        .ok(),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap(),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap(),
                })
            }
        )?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Gets articles by tag
    pub fn get_articles_by_tag(&self, tag_id: &TagId) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT a.id, a.feed_id, a.title, a.url, a.author, a.content, a.summary, a.published_at, a.created_at, a.updated_at
             FROM articles a
             JOIN article_tags at ON a.id = at.article_id
             WHERE at.tag_id = ?
             ORDER BY a.published_at DESC"
        )?;

        let articles = stmt.query_map([tag_id.to_string()], |row| {
            Ok(Article {
                id: ArticleId::from_str(row.get::<_, String>(0)?).unwrap(),
                feed_id: FeedId::from_str(row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                url: row.get(3)?,
                author: Some(row.get(4)?),
                content: Some(row.get(5)?),
                summary: Some(row.get(6)?),
                published_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.with_timezone(&Utc))
                    .ok(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Marks an article as read
    pub fn mark_as_read(&self, article_id: &ArticleId) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "UPDATE articles
             SET is_read = true, updated_at = ?
             WHERE id = ?"
        )?;

        stmt.execute([
            &Utc::now().to_rfc3339(),
            &article_id.to_string(),
        ])?;

        Ok(())
    }

    /// Adds an article to favorites
    pub fn add_to_favorites(&self, article_id: &ArticleId) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT INTO article_favorites (article_id, created_at)
             VALUES (?, ?)"
        )?;

        stmt.execute([
            &article_id.to_string(),
            &Utc::now().to_rfc3339(),
        ])?;

        Ok(())
    }

    /// Removes an article from favorites
    pub fn remove_from_favorites(&self, article_id: &ArticleId) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "DELETE FROM article_favorites
             WHERE article_id = ?"
        )?;

        stmt.execute([&article_id.to_string()])?;

        Ok(())
    }

    /// Checks if an article is in favorites
    pub fn is_favorite(&self, article_id: &ArticleId) -> Result<bool> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT COUNT(*)
             FROM article_favorites
             WHERE article_id = ?"
        )?;

        let count: i64 = stmt.query_row([&article_id.to_string()], |row| row.get(0))?;

        Ok(count > 0)
    }

    /// Gets unread articles count
    pub fn get_unread_count(&self) -> Result<i64> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT COUNT(*)
             FROM articles
             WHERE is_read = false"
        )?;

        let count: i64 = stmt.query_row([], |row| row.get(0))?;

        Ok(count)
    }

    /// Gets favorite articles count
    pub fn get_favorites_count(&self) -> Result<i64> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT COUNT(*)
             FROM article_favorites"
        )?;

        let count: i64 = stmt.query_row([], |row| row.get(0))?;

        Ok(count)
    }
}

// Implement ToSql for ArticleId
impl rusqlite::ToSql for ArticleId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(self.to_string()))
    }
}
