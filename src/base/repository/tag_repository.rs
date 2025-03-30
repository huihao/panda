use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::models::tag::{Tag, TagId};

/// Trait defining the interface for tag repository implementations
pub trait TagRepository: Send + Sync {
    /// Saves a tag to the repository
    fn save_tag(&self, tag: &Tag) -> Result<()>;
    
    /// Retrieves a tag by its name
    fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>>;
    
    /// Retrieves all tags from the repository
    fn get_all_tags(&self) -> Result<Vec<Tag>>;
    
    /// Updates an existing tag
    fn update_tag(&self, tag: &Tag) -> Result<()>;
    
    /// Deletes a tag by its ID
    fn delete_tag(&self, id: &TagId) -> Result<()>;
    
    /// Searches for tags matching the given query
    fn search_tags(&self, query: &str) -> Result<Vec<Tag>>;
    
    /// Retrieves tags associated with articles published within the given date range
    fn get_tags_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Tag>>;
    
    /// Retrieves all tags associated with a specific article
    fn get_article_tags(&self, article_id: &str) -> Result<Vec<Tag>>;
    
    /// Associates a tag with an article
    fn add_tag_to_article(&self, article_id: &str, tag_id: &TagId) -> Result<()>;
    
    /// Removes a tag association from an article
    fn remove_tag_from_article(&self, article_id: &str, tag_id: &TagId) -> Result<()>;
    
    /// Retrieves all article IDs associated with a specific tag
    fn get_articles_with_tag(&self, tag_id: &TagId) -> Result<Vec<String>>;
    
    /// Retrieves the most frequently used tags, limited by the given count
    fn get_most_used_tags(&self, limit: usize) -> Result<Vec<Tag>>;
} 