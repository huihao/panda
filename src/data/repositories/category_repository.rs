use std::sync::Arc;
use rusqlite::Connection;
use anyhow::Result;
use async_trait::async_trait;

use crate::models::category::{Category, CategoryId};
use crate::base::repository::CategoryRepository;

pub struct SqliteCategoryRepository {
    connection: Arc<Connection>,
}

impl SqliteCategoryRepository {
    pub fn new(connection: Arc<Connection>) -> Self {
        Self { connection }
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
        let mut stmt = self.connection.prepare(
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
        let mut stmt = self.connection.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| self.map_row(row))?;
        let categories = rows.collect::<Result<Vec<_>>>()?;
        Ok(categories)
    }

    async fn get_categories_by_parent(&self, parent_id: &Option<CategoryId>) -> Result<Vec<Category>> {
        let mut stmt = match parent_id {
            Some(id) => {
                self.connection.prepare(
                    "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
                     FROM categories 
                     WHERE parent_id = ? 
                     ORDER BY name"
                )?
            },
            None => {
                self.connection.prepare(
                    "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
                     FROM categories 
                     WHERE parent_id IS NULL 
                     ORDER BY name"
                )?
            }
        };

        let rows = if let Some(id) = parent_id {
            stmt.query_map([id.to_string()], |row| self.map_row(row))?
        } else {
            stmt.query_map([], |row| self.map_row(row))?
        };

        let categories = rows.collect::<Result<Vec<_>>>()?;
        Ok(categories)
    }

    async fn get_root_categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             WHERE parent_id IS NULL 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| self.map_row(row))?;
        let categories = rows.collect::<Result<Vec<_>>>()?;
        Ok(categories)
    }

    async fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, description, parent_id, is_expanded, created_at, updated_at 
             FROM categories 
             WHERE parent_id = ? 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([parent_id.to_string()], |row| self.map_row(row))?;
        let categories = rows.collect::<Result<Vec<_>>>()?;
        Ok(categories)
    }

    async fn save_category(&self, category: &Category) -> Result<()> {
        self.connection.execute(
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
        self.connection.execute(
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
        self.connection.execute("DELETE FROM categories WHERE id = ?", [id.to_string()])?;
        Ok(())
    }
}