use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rusqlite::types::{FromSql, ToSql, ToSqlOutput, ValueRef, FromSqlResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub description: String,
    pub parent_id: Option<CategoryId>,
    pub is_expanded: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Category {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: CategoryId::new(),
            name,
            description: String::new(),
            parent_id: None,
            is_expanded: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_parent(mut self, parent_id: CategoryId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.is_expanded = expanded;
        self.updated_at = Utc::now();
    }
}

impl CategoryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl FromSql for CategoryId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        String::column_result(value).map(CategoryId)
    }
}

impl ToSql for CategoryId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.clone()))
    }
}

impl From<String> for CategoryId {
    fn from(s: String) -> Self {
        CategoryId(s)
    }
}