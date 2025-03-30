use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, ConnectionManager, Pool, Row};

use crate::base::repository::TagRepository;
use crate::models::tag::{Tag, TagId};
use crate::models::article::Article;
use crate::data::database::SqliteConnectionManager;
use r2d2::Pool;

/// SQLite implementation of the TagRepository trait
pub struct SqliteTagRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl SqliteTagRepository {
    /// Creates a new SQLite tag repository
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }
    
    /// Maps a database row to a Tag
    fn map_row(row: &Row) -> Result<Tag> {
        Ok(Tag {
            id: TagId(row.get(0)?),
            name: row.get(1)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)?.with_timezone(&Utc),
        })
    }
}

impl TagRepository for SqliteTagRepository {
    fn create_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO tags (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                tag.id.0,
                tag.name,
                tag.description,
                tag.created_at.to_rfc3339(),
                tag.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_tag_by_id(&self, id: &str) -> Result<Option<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM tags WHERE id = ?")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_row(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at
             FROM tags
             ORDER BY name",
        )?;
        let rows = stmt.query_map(params![], Self::map_row)?;
        let mut tags = Vec::new();
        for tag in rows {
            tags.push(tag??);
        }
        Ok(tags)
    }

    fn get_tags_by_name(&self, name: &str) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at
             FROM tags
             WHERE name LIKE ?1
             ORDER BY name",
        )?;
        let rows = stmt.query_map(params![format!("%{}%", name)], Self::map_row)?;
        let mut tags = Vec::new();
        for tag in rows {
            tags.push(tag??);
        }
        Ok(tags)
    }

    fn update_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE tags
             SET name = ?1, description = ?2, updated_at = ?3
             WHERE id = ?4",
            params![
                tag.name,
                tag.description,
                tag.updated_at.to_rfc3339(),
                tag.id.0,
            ],
        )?;
        Ok(())
    }

    fn delete_tag(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM tags WHERE id = ?")?;
        stmt.execute(params![id])?;
        Ok(())
    }
    
    fn search_tags(&self, query: &str) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM tags
            WHERE name LIKE ? OR description LIKE ?
            ORDER BY name"
        )?;
        
        let pattern = format!("%{}%", query);
        let mut tags = Vec::new();
        let rows = stmt.query([&pattern, &pattern])?;
        
        for row in rows {
            tags.push(Self::map_row(&row?)?);
        }
        
        Ok(tags)
    }
    
    fn get_tags_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT t.* FROM tags t
            JOIN article_tags at ON t.id = at.tag_id
            JOIN articles a ON at.article_id = a.id
            WHERE a.published_at BETWEEN ? AND ?
            ORDER BY t.name"
        )?;
        
        let mut tags = Vec::new();
        let rows = stmt.query([
            &start.to_rfc3339(),
            &end.to_rfc3339(),
        ])?;
        
        for row in rows {
            tags.push(Self::map_row(&row?)?);
        }
        
        Ok(tags)
    }
    
    fn get_article_tags(&self, article_id: &str) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.* FROM tags t
            JOIN article_tags at ON t.id = at.tag_id
            WHERE at.article_id = ?
            ORDER BY t.name"
        )?;
        
        let mut tags = Vec::new();
        let rows = stmt.query([article_id])?;
        
        for row in rows {
            tags.push(Self::map_row(&row?)?);
        }
        
        Ok(tags)
    }
    
    fn add_tag_to_article(&self, article_id: &str, tag_id: &TagId) -> Result<()> {
        let conn = self.pool.get()?;
        
        conn.execute(
            "INSERT OR IGNORE INTO article_tags (article_id, tag_id, created_at)
            VALUES (?, ?, ?)",
            [
                article_id,
                &tag_id.0,
                &Utc::now().to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }
    
    fn remove_tag_from_article(&self, article_id: &str, tag_id: &TagId) -> Result<()> {
        let conn = self.pool.get()?;
        
        conn.execute(
            "DELETE FROM article_tags WHERE article_id = ? AND tag_id = ?",
            [article_id, &tag_id.0],
        )?;
        
        Ok(())
    }
    
    fn get_articles_with_tag(&self, tag_id: &str) -> Result<Vec<Article>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT a.* FROM articles a 
             JOIN article_tags at ON a.id = at.article_id 
             WHERE at.tag_id = ?"
        )?;
        let mut rows = stmt.query(params![tag_id])?;
        let mut articles = Vec::new();
        while let Some(row) = rows.next()? {
            articles.push(Self::map_article_row(row)?);
        }
        Ok(articles)
    }
    
    fn get_most_used_tags(&self, limit: usize) -> Result<Vec<(Tag, i64)>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.*, COUNT(at.article_id) as usage_count 
             FROM tags t 
             LEFT JOIN article_tags at ON t.id = at.tag_id 
             GROUP BY t.id 
             ORDER BY usage_count DESC 
             LIMIT ?"
        )?;
        let mut rows = stmt.query(params![limit])?;
        let mut tags = Vec::new();
        while let Some(row) = rows.next()? {
            let tag = Self::map_row(row)?;
            let usage_count: i64 = row.get("usage_count")?;
            tags.push((tag, usage_count));
        }
        Ok(tags)
    }

    fn save_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO tags (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)"
        )?;

        stmt.execute(params![
            tag.id.0,
            tag.name,
            tag.created_at.to_rfc3339(),
            tag.updated_at.to_rfc3339(),
        ])?;

        Ok(())
    }

    fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, updated_at FROM tags WHERE name = ?"
        )?;

        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_row(row)?))
        } else {
            Ok(None)
        }
    }

    fn map_article_row(row: &Row) -> Result<Article> {
        Ok(Article {
            id: row.get(0)?,
            title: row.get(1)?,
            feed_id: row.get(2)?,
            url: row.get(3)?,
            author: row.get(4)?,
            content: row.get(5)?,
            summary: row.get(6)?,
            published_at: row.get(7).map(|date: String| DateTime::parse_from_rfc3339(&date).ok().map(|dt| dt.with_timezone(&Utc))).flatten(),
            is_favorite: row.get(8)?,
            is_read: row.get(9)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?)?.with_timezone(&Utc),
        })
    }
}