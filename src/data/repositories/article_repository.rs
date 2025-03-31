use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use chrono::{DateTime, Utc};
use anyhow::Result;
use async_trait::async_trait;

use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::models::feed::FeedId;
use crate::models::category::CategoryId;
use crate::models::tag::TagId;
use crate::base::repository::ArticleRepository;

pub struct SqliteArticleRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteArticleRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    fn map_row(&self, row: &rusqlite::Row) -> Result<Article> {
        Ok(Article {
            id: row.get::<_, String>(0)?.into(),
            feed_id: row.get::<_, String>(1)?.into(),
            title: row.get(2)?,
            url: row.get(3)?,
            author: row.get(4)?,
            content: row.get(5)?,
            summary: row.get(6)?,
            published_at: row.get(7)?,
            read_status: ReadStatus::from_str(&row.get::<_, String>(8)?).unwrap_or(ReadStatus::Unread),
            is_favorited: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

#[async_trait]
impl ArticleRepository for SqliteArticleRepository {
    async fn get_article(&self, id: &ArticleId) -> Result<Option<Article>> {
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary, 
                    published_at, read_status, is_favorited, created_at, updated_at 
             FROM articles 
             WHERE id = ?"
        )?;

        let mut rows = stmt.query([id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.map_row(row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_all_articles(&self) -> Result<Vec<Article>> {
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT id, feed_id, title, url, author, content, summary,
                    published_at, read_status, is_favorited, created_at, updated_at 
             FROM articles 
             ORDER BY published_at DESC"
        )?;

        let rows = stmt.query_map([], |row| Ok(self.map_row(row)))?;
        let articles = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    async fn get_articles_by_feed(&self, feed_id: &FeedId) -> Result<Vec<Article>> {
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT id, feed_id, title, url, author, content, summary,
                    published_at, read_status, is_favorited, created_at, updated_at 
             FROM articles 
             WHERE feed_id = ? 
             ORDER BY published_at DESC"
        )?;

        let rows = stmt.query_map([feed_id.to_string()], |row| Ok(self.map_row(row)))?;
        let articles = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    async fn get_articles_by_category(&self, category_id: &CategoryId) -> Result<Vec<Article>> {
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT a.id, a.feed_id, a.category_id, a.title, a.url, a.author, a.content, 
                    a.summary, a.published_at, a.read_status, a.is_favorited, 
                    a.created_at, a.updated_at 
             FROM articles a 
             JOIN feeds f ON a.feed_id = f.id 
             WHERE f.category_id = ? 
             ORDER BY a.published_at DESC"
        )?;

        let rows = stmt.query_map([category_id.to_string()], |row| Ok(self.map_row(row)))?;
        let articles = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    async fn get_articles_by_tag(&self, tag_id: &TagId) -> Result<Vec<Article>> {
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT a.id, a.feed_id, a.category_id, a.title, a.url, a.author, a.content, 
                    a.summary, a.published_at, a.read_status, a.is_favorited, 
                    a.created_at, a.updated_at 
             FROM articles a 
             JOIN article_tags at ON a.id = at.article_id 
             WHERE at.tag_id = ? 
             ORDER BY a.published_at DESC"
        )?;

        let rows = stmt.query_map([tag_id.to_string()], |row| Ok(self.map_row(row)))?;
        let articles = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    async fn get_unread_articles(&self) -> Result<Vec<Article>> {
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary,
                    published_at, read_status, is_favorited, created_at, updated_at 
             FROM articles 
             WHERE read_status = 'unread' 
             ORDER BY published_at DESC"
        )?;

        let rows = stmt.query_map([], |row| Ok(self.map_row(row)))?;
        let articles = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    async fn get_favorite_articles(&self) -> Result<Vec<Article>> {
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary,
                    published_at, read_status, is_favorited, created_at, updated_at 
             FROM articles 
             WHERE is_favorited = true 
             ORDER BY published_at DESC"
        )?;

        let rows = stmt.query_map([], |row| Ok(self.map_row(row)))?;
        let articles = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    async fn search_articles(&self, query: &str) -> Result<Vec<Article>> {
        let search_term = format!("%{}%", query);
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary,
                    published_at, read_status, is_favorited, created_at, updated_at 
             FROM articles 
             WHERE title LIKE ? OR content LIKE ? OR summary LIKE ? 
             ORDER BY published_at DESC"
        )?;

        let rows = stmt.query_map([&search_term, &search_term, &search_term], |row| Ok(self.map_row(row)))?;
        let articles = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    async fn get_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>> {
        let mut stmt = self.connection.lock().unwrap().prepare(
            "SELECT id, feed_id, category_id, title, url, author, content, summary,
                    published_at, read_status, is_favorited, created_at, updated_at 
             FROM articles 
             WHERE published_at BETWEEN ? AND ? 
             ORDER BY published_at DESC"
        )?;

        let rows = stmt.query_map([start, end], |row| Ok(self.map_row(row)))?;
        let articles = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }

    async fn save_article(&self, article: &Article) -> Result<()> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO articles (
                id, feed_id, title, url, author, content, summary, published_at,
                read_status, is_favorited, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                article.id.to_string(),
                article.feed_id.to_string(),
                article.title,
                article.url.to_string(),
                article.author,
                article.content,
                article.summary,
                article.published_at,
                article.read_status.to_string(),
                article.is_favorited,
                article.created_at,
                article.updated_at,
            ],
        )?;
        Ok(())
    }

    async fn update_article(&self, article: &Article) -> Result<()> {
        self.connection.lock().unwrap().execute(
            "UPDATE articles SET
                feed_id = ?,
                title = ?,
                url = ?,
                author = ?,
                content = ?,
                summary = ?,
                published_at = ?,
                read_status = ?,
                is_favorited = ?,
                updated_at = ?
            WHERE id = ?",
            rusqlite::params![
                article.feed_id.to_string(),
                article.title,
                article.url.to_string(),
                article.author,
                article.content,
                article.summary,
                article.published_at,
                article.read_status.to_string(),
                article.is_favorited,
                article.updated_at,
                article.id.to_string(),
            ],
        )?;
        Ok(())
    }

    async fn delete_article(&self, id: &ArticleId) -> Result<()> {
        self.connection.lock().unwrap().execute("DELETE FROM articles WHERE id = ?", [id.to_string()])?;
        Ok(())
    }
}
