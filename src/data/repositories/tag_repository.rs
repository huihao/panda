use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use anyhow::Result;
use async_trait::async_trait;

use crate::models::article::ArticleId;
use crate::models::tag::{Tag, TagId};
use crate::base::repository::TagRepository;

pub struct SqliteTagRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteTagRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    fn map_row(&self, row: &rusqlite::Row) -> Result<Tag> {
        Ok(Tag {
            id: row.get::<_, String>(0)?.into(),
            name: row.get(1)?,
            description: row.get(2)?,
            color: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
}

#[async_trait]
impl TagRepository for SqliteTagRepository {
    async fn get_tag_by_id(&self, id: &TagId) -> Result<Option<Tag>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, color, created_at, updated_at 
             FROM tags 
             WHERE id = ?"
        )?;

        let mut rows = stmt.query([id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.map_row(row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, color, created_at, updated_at 
             FROM tags 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| Ok(self.map_row(row)))?;
        let tags = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    async fn get_tags_by_article(&self, article_id: &ArticleId) -> Result<Vec<Tag>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.description, t.color, t.created_at, t.updated_at 
             FROM tags t 
             JOIN article_tags at ON t.id = at.tag_id 
             WHERE at.article_id = ? 
             ORDER BY t.name"
        )?;

        let rows = stmt.query_map([article_id.to_string()], |row| Ok(self.map_row(row)))?;
        let tags = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    async fn save_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO tags (id, name, description, color, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                tag.id.to_string(),
                tag.name,
                tag.description,
                tag.color,
                tag.created_at,
                tag.updated_at,
            ],
        )?;
        Ok(())
    }

    async fn update_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "UPDATE tags SET 
                name = ?, 
                description = ?,
                color = ?,
                updated_at = ?
             WHERE id = ?",
            rusqlite::params![
                tag.name,
                tag.description,
                tag.color,
                tag.updated_at,
                tag.id.to_string(),
            ],
        )?;
        Ok(())
    }

    async fn delete_tag(&self, id: &TagId) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM tags WHERE id = ?", [id.to_string()])?;
        Ok(())
    }

    async fn add_tag_to_article(&self, article_id: &ArticleId, tag_id: &TagId) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO article_tags (article_id, tag_id, created_at)
             VALUES (?, ?, datetime('now'))",
            rusqlite::params![article_id.to_string(), tag_id.to_string()],
        )?;
        Ok(())
    }

    async fn remove_tag_from_article(&self, article_id: &ArticleId, tag_id: &TagId) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM article_tags WHERE article_id = ? AND tag_id = ?",
            rusqlite::params![article_id.to_string(), tag_id.to_string()],
        )?;
        Ok(())
    }
}