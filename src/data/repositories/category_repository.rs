use anyhow::Result;
use std::sync::Arc;
use rusqlite::{Connection, params, Row, OptionalExtension};
use log::{info, error};
use chrono::{DateTime, Utc};

use crate::base::repository::CategoryRepository;
use crate::models::feed::{Category, CategoryId};
use crate::data::types::DateTimeWrapper;

/// SQLite-based category repository implementation
pub struct SqliteCategoryRepository {
    connection: Arc<Connection>,
}

impl SqliteCategoryRepository {
    /// Creates a new SQLite category repository
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Arc::new(connection),
        }
    }
    
    /// Maps a database row to a Category
    fn map_row(row: &Row) -> Result<Category, rusqlite::Error> {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let description: Option<String> = row.get(2)?;
        let parent_id: Option<String> = row.get(3)?;
        let created_at: DateTimeWrapper = row.get(4)?;
        let updated_at: DateTimeWrapper = row.get(5)?;

        Ok(Category {
            id: CategoryId(id),
            name,
            description,
            parent_id: parent_id.map(CategoryId),
            created_at: created_at.0,
            updated_at: updated_at.0,
        })
    }
}

impl CategoryRepository for SqliteCategoryRepository {
    fn get_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, description, parent_id, created_at, updated_at 
             FROM categories WHERE id = ?"
        )?;
        
        let category = stmt.query_row(params![id.0], Self::map_row).optional()?;
        Ok(category)
    }
    
    fn get_category_by_name(&self, name: &str) -> Result<Option<Category>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, description, parent_id, created_at, updated_at 
             FROM categories WHERE name = ?"
        )?;
        
        let category = stmt.query_row(params![name], Self::map_row).optional()?;
        Ok(category)
    }
    
    fn get_all_categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, description, parent_id, created_at, updated_at 
             FROM categories ORDER BY name"
        )?;
        
        let categories = stmt.query_map([], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
    
    fn get_categories_by_parent(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, description, parent_id, created_at, updated_at 
             FROM categories WHERE parent_id = ? ORDER BY name"
        )?;
        
        let categories = stmt.query_map(params![parent_id.0], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
    
    fn save_category(&self, category: &Category) -> Result<()> {
        let mut stmt = self.connection.prepare(
            "INSERT OR REPLACE INTO categories (
                id, name, description, parent_id, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)"
        )?;

        let created_at = DateTimeWrapper(category.created_at);
        let updated_at = DateTimeWrapper(category.updated_at);

        stmt.execute(params![
            category.id.0,
            category.name,
            category.description,
            category.parent_id.as_ref().map(|id| id.0.clone()),
            created_at,
            updated_at,
        ])?;

        Ok(())
    }
    
    fn update_category(&self, category: &Category) -> Result<()> {
        let mut stmt = self.connection.prepare(
            "UPDATE categories SET 
                name = ?,
                description = ?,
                parent_id = ?,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = ?"
        )?;

        stmt.execute(params![
            category.name,
            category.description,
            category.parent_id.as_ref().map(|id| id.0.clone()),
            category.id.0,
        ])?;

        Ok(())
    }
    
    fn delete_category(&self, id: &CategoryId) -> Result<()> {
        let mut stmt = self.connection.prepare(
            "DELETE FROM categories WHERE id = ?"
        )?;
        
        stmt.execute(params![id.0])?;
        Ok(())
    }
    
    fn search_categories(&self, query: &str) -> Result<Vec<Category>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, description, parent_id, created_at, updated_at 
             FROM categories WHERE name LIKE ? OR description LIKE ? ORDER BY name"
        )?;
        
        let pattern = format!("%{}%", query);
        let categories = stmt.query_map(params![pattern, pattern], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
    
    fn get_category_tree(&self) -> Result<Vec<Category>> {
        let mut stmt = self.connection.prepare(
            "WITH RECURSIVE category_tree AS (
                SELECT id, name, description, parent_id, created_at, updated_at, 1 as level
                FROM categories
                WHERE parent_id IS NULL
                UNION ALL
                SELECT c.id, c.name, c.description, c.parent_id, c.created_at, c.updated_at, ct.level + 1
                FROM categories c
                JOIN category_tree ct ON c.parent_id = ct.id
            )
            SELECT id, name, description, parent_id, created_at, updated_at
            FROM category_tree
            ORDER BY level, name"
        )?;
        
        let categories = stmt.query_map([], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
}