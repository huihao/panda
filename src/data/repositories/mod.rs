// Make each repository module public to enable access to the implementations
// This follows the Open/Closed Principle by making these modules extensible without modification
pub mod article_repository;
pub mod category_repository;
pub mod feed_repository;
pub mod tag_repository;

// Re-export the concrete implementations to provide a cleaner public API
// This follows the Interface Segregation Principle by exposing only what clients need
pub use article_repository::SqliteArticleRepository;
pub use category_repository::SqliteCategoryRepository;
pub use feed_repository::SqliteFeedRepository;
pub use tag_repository::SqliteTagRepository;