use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rusqlite::types::{FromSql, ToSql, ToSqlOutput, ValueRef, FromSqlResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub description: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: TagId::new(),
            name,
            description: String::new(),
            color: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

impl TagId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl FromSql for TagId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        String::column_result(value).map(TagId)
    }
}

impl ToSql for TagId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.clone()))
    }
}

impl From<String> for TagId {
    fn from(s: String) -> Self {
        TagId(s)
    }
}