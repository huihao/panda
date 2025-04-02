use std::sync::Arc;
use rusqlite::Connection;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::models::category::{Category, CategoryId};
use crate::base::repository::CategoryRepository;
use crate::data::database::ConnectionPool;

pub struct SqliteCategoryRepository {
    connection_pool: Arc<ConnectionPool>,
}

impl SqliteCategoryRepository {
    pub fn new(connection_pool: Arc<ConnectionPool>) -> Self {
        Self { connection_pool }
    }

    fn map_row(&self, row: &rusqlite::Row) -> Result<Category> {
        Ok(Category {
            id: row.get::<_, String>(0)?.into(),
            name: row.get(1)?,
            description: row.get(2)?,
            parent_id: row.get::<_, Option<String>>(3)?.map(|s| s.into()),
            is_expanded: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}

#[async_trait]
impl CategoryRepository for SqliteCategoryRepository {
    async fn get_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>> {
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             WHERE id = ?"
        )?;

        let mut rows = stmt.query([id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.map_row(row)?))
        } else {
            Ok(None)
        }
    }
    
    async fn get_all_categories(&self) -> Result<Vec<Category>> {
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| Ok(self.map_row(row)))?;
        let categories = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
    
    async fn get_categories_by_parent(&self, parent_id: &Option<CategoryId>) -> Result<Vec<Category>> {
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection_pool.get()?;
        
        let categories = match parent_id {
            Some(id) => {
                let mut stmt = conn.prepare(
                    "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
                     FROM categories 
                     WHERE parent_id = ? 
                     ORDER BY name"
                )?;
                let rows = stmt.query_map([id.to_string()], |row| Ok(self.map_row(row)))?;
                rows.collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?
            },
            None => {
                let mut stmt = conn.prepare(
                    "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
                     FROM categories 
                     WHERE parent_id IS NULL 
                     ORDER BY name"
                )?;
                let rows = stmt.query_map([], |row| Ok(self.map_row(row)))?;
                rows.collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?
            }
        };
        
        Ok(categories)
    }
    
    async fn get_root_categories(&self) -> Result<Vec<Category>> {
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             WHERE parent_id IS NULL 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| Ok(self.map_row(row)))?;
        let categories = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
    
    async fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             WHERE parent_id = ? 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([parent_id.to_string()], |row| Ok(self.map_row(row)))?;
        let categories = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
    
    async fn save_category(&self, category: &Category) -> Result<()> {
        self.connection_pool.get()?.execute(
            "INSERT INTO categories (
                id, name, description, parent_id, is_expanded, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                category.id.to_string(),
                category.name,
                category.description,
                category.parent_id.as_ref().map(|id| id.to_string()),
                category.is_expanded,
                category.created_at,
                category.updated_at,
            ],
        )?;
        Ok(())
    }
    
    async fn update_category(&self, category: &Category) -> Result<()> {
        self.connection_pool.get()?.execute(
            "UPDATE categories SET
                name = ?,
                description = ?,
                parent_id = ?,
                is_expanded = ?,
                updated_at = ?
            WHERE id = ?",
            rusqlite::params![
                category.name,
                category.description,
                category.parent_id.as_ref().map(|id| id.to_string()),
                category.is_expanded,
                category.updated_at,
                category.id.to_string(),
            ],
        )?;
        Ok(())
    }
    
    async fn delete_category(&self, id: &CategoryId) -> Result<()> {
        self.connection_pool.get()?.execute("DELETE FROM categories WHERE id = ?", [id.to_string()])?;
        Ok(())
    }
    
    async fn search_categories(&self, name: &str) -> Result<Vec<Category>> {
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection_pool.get()?;
        let search_term = format!("%{}%", name);
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             WHERE name LIKE ? 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([&search_term], |row| Ok(self.map_row(row)))?;
        let categories = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }

    async fn get_recently_updated_categories(&self, limit: usize) -> Result<Vec<Category>> {
        // Store the connection lock in a variable to extend its lifetime
        let conn = self.connection_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             ORDER BY updated_at DESC 
             LIMIT ?"
        )?;

        let rows = stmt.query_map([limit as i64], |row| Ok(self.map_row(row)))?;
        let categories = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
    
    async fn get_category_hierarchy(&self) -> Result<Vec<Category>> {
        // First get all categories
        let all_categories = self.get_all_categories().await?;
        
        // Build hierarchy (this is a simple implementation)
        // In a real application, you might want a more sophisticated approach
        Ok(all_categories)
    }

    async fn get_categories_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Category>> {
        // 锁定连接
        let conn = self.connection_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             WHERE created_at BETWEEN ? AND ? 
             ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map([start, end], |row| Ok(self.map_row(row)))?;
        let categories = rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
}