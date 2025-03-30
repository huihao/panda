use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid;

/// A unique identifier for a tag
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagId(pub String);

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a tag that can be associated with articles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// The unique identifier of the tag
    pub id: TagId,
    
    /// The name of the tag
    pub name: String,
    
    /// An optional description of the tag
    pub description: Option<String>,
    
    /// An optional color for the tag (in hex format)
    pub color: Option<String>,
    
    /// When the tag was created
    pub created_at: DateTime<Utc>,
    
    /// When the tag was last updated
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    /// Creates a new tag with the given name
    pub fn new(name: String) -> Self {
        Self {
            id: TagId(uuid::Uuid::new_v4().to_string()),
            name,
            description: None,
            color: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Sets the description of the tag
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }
    
    /// Sets the color of the tag
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self.updated_at = Utc::now();
        self
    }
    
    /// Updates the name of the tag
    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }
    
    /// Updates the description of the tag
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }
    
    /// Updates the color of the tag
    pub fn update_color(&mut self, color: Option<String>) {
        self.color = color;
        self.updated_at = Utc::now();
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tag_creation() {
        let name = "Test Tag".to_string();
        let tag = Tag::new(name.clone());
        
        assert_eq!(tag.name, name);
        assert!(tag.description.is_none());
        assert!(tag.color.is_none());
    }
    
    #[test]
    fn test_tag_with_description() {
        let description = "Test Description".to_string();
        let tag = Tag::new("Test Tag".to_string())
            .with_description(description.clone());
        
        assert_eq!(tag.description, Some(description));
    }
    
    #[test]
    fn test_tag_with_color() {
        let color = "#FF0000".to_string();
        let tag = Tag::new("Test Tag".to_string())
            .with_color(color.clone());
        
        assert_eq!(tag.color, Some(color));
    }
    
    #[test]
    fn test_tag_update() {
        let mut tag = Tag::new("Test Tag".to_string());
        
        tag.update_name("Updated Tag".to_string());
        assert_eq!(tag.name, "Updated Tag");
        
        tag.update_description(Some("Updated Description".to_string()));
        assert_eq!(tag.description, Some("Updated Description".to_string()));
        
        tag.update_color(Some("#00FF00".to_string()));
        assert_eq!(tag.color, Some("#00FF00".to_string()));
    }
} 