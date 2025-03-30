use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fmt;

/// Unique identifier for a category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(pub String);

impl fmt::Display for CategoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Category model for organizing feeds and articles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier
    pub id: CategoryId,
    /// Category name
    pub name: String,
    /// Category description
    pub description: Option<String>,
    /// Parent category ID (for hierarchical categories)
    pub parent_id: Option<CategoryId>,
    /// Whether the category is expanded in the UI
    pub is_expanded: bool,
    /// When the category was created
    pub created_at: DateTime<Utc>,
    /// When the category was last updated
    pub updated_at: DateTime<Utc>,
}

impl Category {
    /// Creates a new category
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: CategoryId(Uuid::new_v4().to_string()),
            name,
            description: None,
            parent_id: None,
            is_expanded: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Sets the category's description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the category's parent
    pub fn with_parent_id(mut self, parent_id: CategoryId) -> Self {
        self.parent_id = Some(parent_id);
        self.updated_at = Utc::now();
        self
    }
    
    /// Toggles whether the category is expanded in the UI
    pub fn toggle_expanded(&mut self) {
        self.is_expanded = !self.is_expanded;
        self.updated_at = Utc::now();
    }
    
    /// Updates the category's last update time
    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now();
    }
    
    /// Updates the name of the category
    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }
    
    /// Updates the description of the category
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }
    
    /// Updates the parent ID of the category
    pub fn update_parent_id(&mut self, parent_id: Option<CategoryId>) {
        self.parent_id = parent_id;
        self.updated_at = Utc::now();
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl CategoryId {
    pub fn new() -> Self {
        CategoryId(Uuid::new_v4().to_string())
    }

    pub fn root() -> Self {
        Self("root".to_string())
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_category_creation() {
        let name = "Test Category".to_string();
        let category = Category::new(name.clone());
        
        assert_eq!(category.name, name);
        assert!(category.description.is_none());
        assert!(category.parent_id.is_none());
        assert!(category.is_expanded);
    }
    
    #[test]
    fn test_category_with_description() {
        let description = "Test Description".to_string();
        let category = Category::new("Test Category".to_string())
            .with_description(description.clone());
        
        assert_eq!(category.description, Some(description));
    }
    
    #[test]
    fn test_category_with_parent() {
        let parent_id = CategoryId(Uuid::new_v4().to_string());
        let category = Category::new("Test Category".to_string())
            .with_parent_id(parent_id);
        
        assert_eq!(category.parent_id, Some(parent_id));
    }
    
    #[test]
    fn test_category_toggle_expanded() {
        let mut category = Category::new("Test Category".to_string());
        
        category.toggle_expanded();
        assert!(!category.is_expanded);
        
        category.toggle_expanded();
        assert!(category.is_expanded);
    }
    
    #[test]
    fn test_category_update() {
        let mut category = Category::new("Test Category".to_string());
        
        category.update_name("Updated Category".to_string());
        assert_eq!(category.name, "Updated Category");
        
        category.update_description(Some("Updated Description".to_string()));
        assert_eq!(category.description, Some("Updated Description".to_string()));
        
        let parent_id = CategoryId(Uuid::new_v4().to_string());
        category.update_parent_id(Some(parent_id));
        assert_eq!(category.parent_id, Some(parent_id));
    }
} 