use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use chrono::{DateTime, Utc};
use anyhow::Result;
use async_trait::async_trait;

use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::models::feed::FeedId;
use crate::models::category::CategoryId;
use crate::models::tag::TagId;
use crate::base::repository_traits::ArticleRepository;

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
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary, 
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
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
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
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
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
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT a.id, a.feed_id, a.title, a.url, a.author, a.content, 
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
    
    async fn get_article_by_url(&self, url: &str) -> Result<Option<Article>> {
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary, 
                    published_at, read_status, is_favorited, created_at, updated_at 
             FROM articles 
             WHERE url = ?"
        )?;

        let mut rows = stmt.query([url])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.map_row(row)?))
        } else {
            Ok(None)
        }
    }
    
    async fn add_tag(&self, article_id: &ArticleId, tag: &str) -> Result<()> {
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        
        // 检查标签是否存在，如果不存在则创建
        let mut stmt = conn.prepare(
            "SELECT id FROM tags WHERE name = ?"
        )?;
        
        let tag_id = match stmt.query_row([tag], |row| Ok(row.get::<_, String>(0)?)) {
            Ok(id) => id,
            Err(_) => {
                // 创建标签
                use uuid::Uuid;
                let id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO tags (id, name, created_at, updated_at) VALUES (?, ?, datetime('now'), datetime('now'))",
                    [&id, tag]
                )?;
                id
            }
        };
        
        // 关联文章和标签
        conn.execute(
            "INSERT OR IGNORE INTO article_tags (article_id, tag_id, created_at) VALUES (?, ?, datetime('now'))",
            [article_id.to_string(), tag_id]
        )?;
        
        Ok(())
    }
    
    async fn remove_tag(&self, article_id: &ArticleId, tag: &str) -> Result<()> {
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        
        // 查找标签ID
        let mut stmt = conn.prepare(
            "SELECT id FROM tags WHERE name = ?"
        )?;
        
        if let Ok(tag_id) = stmt.query_row([tag], |row| Ok(row.get::<_, String>(0)?)) {
            // 删除关联
            conn.execute(
                "DELETE FROM article_tags WHERE article_id = ? AND tag_id = ?",
                [article_id.to_string(), tag_id]
            )?;
        }
        
        Ok(())
    }
    
    async fn get_article_tags(&self, article_id: &ArticleId) -> Result<Vec<String>> {
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT t.name 
             FROM tags t 
             JOIN article_tags at ON t.id = at.tag_id 
             WHERE at.article_id = ? 
             ORDER BY t.name"
        )?;
        
        let rows = stmt.query_map([article_id.to_string()], |row| row.get::<_, String>(0))?;
        let tags = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    async fn get_articles_by_tag(&self, tag: &str) -> Result<Vec<Article>> {
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        
        // 先查找标签ID
        let mut stmt = conn.prepare(
            "SELECT id FROM tags WHERE name = ?"
        )?;
        
        if let Ok(tag_id) = stmt.query_row([tag], |row| Ok(row.get::<_, String>(0)?)) {
            let mut stmt = conn.prepare(
                "SELECT a.id, a.feed_id, a.title, a.url, a.author, a.content, a.summary,
                        a.published_at, a.read_status, a.is_favorited, a.created_at, a.updated_at 
                 FROM articles a 
                 JOIN article_tags at ON a.id = at.article_id 
                 WHERE at.tag_id = ? 
                 ORDER BY a.published_at DESC"
            )?;

            let rows = stmt.query_map([tag_id], |row| Ok(self.map_row(row)))?;
            let articles = rows.collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            Ok(articles)
        } else {
            Ok(vec![]) // 标签不存在，返回空列表
        }
    }
    
    async fn get_unread_articles(&self) -> Result<Vec<Article>> {
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary,
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
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary,
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
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        let search_term = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary,
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
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, url, author, content, summary,
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
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        conn.execute(
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
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        conn.execute(
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
        // 锁定连接以延长其生命周期
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM articles WHERE id = ?", [id.to_string()])?;
        Ok(())
    }
}
