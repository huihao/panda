use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

use crate::models::category::{Category, CategoryId};

/// Trait defining the interface for category repository implementations
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    /// Saves a category to the repository
    async fn save_category(&self, category: &Category) -> Result<()>;
    
    /// Retrieves a category by its ID
    async fn get_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>>;
    
    /// Retrieves all categories from the repository
    async fn get_all_categories(&self) -> Result<Vec<Category>>;
    
    /// Retrieves all root categories (categories without a parent)
    async fn get_root_categories(&self) -> Result<Vec<Category>>;
    
    /// Retrieves all child categories of a specific category
    async fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>>;
    
    /// Updates an existing category
    async fn update_category(&self, category: &Category) -> Result<()>;
    
    /// Deletes a category by its ID
    async fn delete_category(&self, id: &CategoryId) -> Result<()>;
    
    /// Searches for categories matching the given query
    async fn search_categories(&self, query: &str) -> Result<Vec<Category>>;
    
    /// Retrieves categories created within the given date range
    async fn get_categories_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Category>>;
    
    /// Retrieves the most recently updated categories
    async fn get_recently_updated_categories(&self, limit: usize) -> Result<Vec<Category>>;
    
    /// Retrieves the category hierarchy (all categories with their children)
    async fn get_category_hierarchy(&self) -> Result<Vec<Category>>;
    
    /// Retrieves categories by their parent ID
    async fn get_categories_by_parent(&self, parent_id: &Option<CategoryId>) -> Result<Vec<Category>>;
}