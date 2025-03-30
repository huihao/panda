use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Pool, Row};
use uuid::Uuid;
use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::base::repository::CategoryRepository;
use crate::models::category::{Category, CategoryId};

/// SQLite-based category repository implementation
pub struct SqliteCategoryRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl SqliteCategoryRepository {
    /// Creates a new SQLite category repository
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    pub fn init(&self) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                parent_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(parent_id) REFERENCES categories(id)
            )",
            [],
        )?;
        Ok(())
    }

    /// Maps a database row to a Category
    fn map_row(row: &Row) -> Result<Category> {
        Ok(Category {
            id: CategoryId(row.get(0)?),
            name: row.get(1)?,
            parent_id: row.get(2)?,
            is_expanded: row.get(3)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)?.with_timezone(&Utc),
        })
    }
}

impl CategoryRepository for SqliteCategoryRepository {
    fn create_category(&self, category: &Category) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO categories (id, name, description, parent_id, is_expanded, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                category.id.0,
                category.name,
                category.description,
                category.parent_id.as_ref().map(|id| id.0.clone()),
                category.is_expanded,
                category.created_at.to_rfc3339(),
                category.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM categories WHERE id = ?")?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_row(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_all_categories(&self) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at FROM categories"
        )?;

        let mut categories = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            categories.push(Self::map_row(row)?);
        }

        Ok(categories)
    }

    fn get_categories_by_parent(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM categories WHERE parent_id = ?")?;
        let mut rows = stmt.query(params![parent_id.0])?;
        let mut categories = Vec::new();
        while let Some(row) = rows.next()? {
            categories.push(Self::map_row(row)?);
        }
        Ok(categories)
    }

    fn get_root_categories(&self) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM categories WHERE parent_id IS NULL")?;
        let mut rows = stmt.query([])?;
        let mut categories = Vec::new();
        while let Some(row) = rows.next()? {
            categories.push(Self::map_row(row)?);
        }
        Ok(categories)
    }

    fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at FROM categories WHERE parent_id = ?"
        )?;

        let mut categories = Vec::new();
        let mut rows = stmt.query(params![parent_id.0])?;
        while let Some(row) = rows.next()? {
            categories.push(Self::map_row(row)?);
        }

        Ok(categories)
    }

    fn get_categories_by_name(&self, name: &str) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at FROM categories WHERE name LIKE ?"
        )?;

        let mut categories = Vec::new();
        let mut rows = stmt.query(params![format!("%{}%", name)])?;
        while let Some(row) = rows.next()? {
            categories.push(Self::map_row(row)?);
        }

        Ok(categories)
    }

    fn get_recently_updated_categories(&self, limit: usize) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at FROM categories ORDER BY updated_at DESC LIMIT ?"
        )?;

        let mut categories = Vec::new();
        let mut rows = stmt.query(params![limit as i64])?;
        while let Some(row) = rows.next()? {
            categories.push(Self::map_row(row)?);
        }

        Ok(categories)
    }

    fn update_category(&self, category: &Category) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE categories
             SET name = ?1, description = ?2, parent_id = ?3, is_expanded = ?4, updated_at = ?5
             WHERE id = ?6",
            params![
                category.name,
                category.description,
                category.parent_id.as_ref().map(|id| id.0.clone()),
                category.is_expanded,
                category.updated_at.to_rfc3339(),
                category.id.0,
            ],
        )?;
        Ok(())
    }

    fn delete_category(&self, id: &CategoryId) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM categories WHERE id = ?")?;
        stmt.execute(params![id.0])?;
        Ok(())
    }

    fn search_categories(&self, query: &str) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM categories WHERE name LIKE ?")?;
        let pattern = format!("%{}%", query);
        let mut rows = stmt.query(params![pattern])?;
        let mut categories = Vec::new();
        while let Some(row) = rows.next()? {
            categories.push(Self::map_row(row)?);
        }
        Ok(categories)
    }
    
    fn get_categories_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM categories
            WHERE created_at BETWEEN ? AND ?
            ORDER BY name"
        )?;
        
        let mut categories = Vec::new();
        let rows = stmt.query([
            &start.to_rfc3339(),
            &end.to_rfc3339(),
        ])?;
        
        for row in rows {
            categories.push(Self::map_row(row)?);
        }
        
        Ok(categories)
    }
    
    fn get_category_hierarchy(&self) -> Result<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "WITH RECURSIVE category_tree AS (
                SELECT c.*, 1 as level
                FROM categories c
                WHERE parent_id IS NULL
                
                UNION ALL
                
                SELECT c.*, ct.level + 1
                FROM categories c
                JOIN category_tree ct ON c.parent_id = ct.id
            )
            SELECT * FROM category_tree
            ORDER BY level, name"
        )?;
        
        let mut categories = Vec::new();
        let rows = stmt.query([])?;
        
        for row in rows {
            categories.push(Self::map_row(row)?);
        }
        
        Ok(categories)
    }

    fn save_category(&self, category: &Category) -> Result<()> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO categories (id, name, parent_id, is_expanded, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )?;
        stmt.execute(params![
            category.id.0,
            category.name,
            category.parent_id.as_ref().map(|id| id.0.clone()),
            category.is_expanded,
            category.created_at.to_rfc3339(),
            category.updated_at.to_rfc3339(),
        ])?;
        Ok(())
    }
}