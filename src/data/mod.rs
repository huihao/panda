pub mod database;
pub mod repositories;
pub mod types;

// Re-export repository traits and implementations
pub use crate::data::repositories::{
    SqliteFeedRepository,
    SqliteArticleRepository,
    SqliteTagRepository,
};

// Fix the import path to use the external crate reference
pub use crate::base::{ArticleRepository, FeedRepository, TagRepository};

// Re-export database module
pub use crate::data::database::Database;
pub use crate::data::types::*;

pub use database::*;
pub use repositories::*;