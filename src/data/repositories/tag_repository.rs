use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row, OptionalExtension};

// Fix the import path to use the external crate reference
use crate::base::TagRepository;
use crate::models::tag::{Tag, TagId};

pub struct SqliteTagRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteTagRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    fn map_row(&self, row: &Row) -> Result<Tag, rusqlite::Error> {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let created_at: i64 = row.get(2)?;

        Ok(Tag {
            id: TagId(id),
            name,
            created_at: DateTime::from_utc(chrono::NaiveDateTime::from_timestamp(created_at, 0), Utc),
        })
    }
}

impl TagRepository for SqliteTagRepository {
    fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM tags ORDER BY name")?;
        let tags = stmt.query_map([], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }
    
    fn get_tag_by_id(&self, id: &str) -> Result<Option<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM tags WHERE id = ?")?;
        let tag = stmt.query_row([id], |row| self.map_row(row)).optional()?;
        Ok(tag)
    }
    
    fn get_tags_by_name(&self, name: &str) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let search_pattern = format!("%{}%", name);
        let mut stmt = conn.prepare("SELECT * FROM tags WHERE name LIKE ? ORDER BY name")?;
        let tags = stmt.query_map([search_pattern], |row| self.map_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }
    
    fn save_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT INTO tags (id, name, created_at) 
             VALUES (?, ?, ?)"
        )?;

        stmt.execute(params![
            tag.id.0,
            tag.name,
            tag.created_at.timestamp(),
        ])?;

        Ok(())
    }
    
    fn update_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "UPDATE tags SET name = ? WHERE id = ?"
        )?;

        stmt.execute(params![
            tag.name,
            tag.id.0,
        ])?;

        Ok(())
    }
    
    fn delete_tag(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM tags WHERE id = ?")?;
        stmt.execute([id])?;
        Ok(())
    }
}