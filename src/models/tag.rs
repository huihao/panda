use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: TagId(uuid::Uuid::new_v4().to_string()),
            name,
            created_at: now,
        }
    }
} 