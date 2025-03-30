use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::models::category::{Category, CategoryId};

/// Trait defining the interface for category repository implementations
pub trait CategoryRepository: Send + Sync {
    /// Saves a category to the repository
    fn save_category(&self, category: &Category) -> Result<()>;
    
    /// Retrieves a category by its ID
    fn get_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>>;
    
    /// Retrieves all categories from the repository
    fn get_all_categories(&self) -> Result<Vec<Category>>;
    
    /// Retrieves all root categories (categories without a parent)
    fn get_root_categories(&self) -> Result<Vec<Category>>;
    
    /// Retrieves all child categories of a specific category
    fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>>;
    
    /// Updates an existing category
    fn update_category(&self, category: &Category) -> Result<()>;
    
    /// Deletes a category by its ID
    fn delete_category(&self, id: &CategoryId) -> Result<()>;
    
    /// Searches for categories matching the given query
    fn search_categories(&self, query: &str) -> Result<Vec<Category>>;
    
    /// Retrieves categories created within the given date range
    fn get_categories_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Category>>;
    
    /// Retrieves the most recently updated categories
    fn get_recently_updated_categories(&self, limit: usize) -> Result<Vec<Category>>;
    
    /// Retrieves the category hierarchy (all categories with their children)
    fn get_category_hierarchy(&self) -> Result<Vec<Category>>;
} 