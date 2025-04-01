// Export repository traits
mod article_repository;
mod category_repository;
mod feed_repository;
mod tag_repository;

pub use article_repository::ArticleRepository;
pub use category_repository::CategoryRepository;
pub use feed_repository::FeedRepository;
pub use tag_repository::TagRepository;