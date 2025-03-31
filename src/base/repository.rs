use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use crate::models::{
    Feed, FeedId,
    feed::{Category, CategoryId},
    Article, ArticleId,
    Tag
};

/// Repository interface for feed operations
pub trait FeedRepository: Send + Sync {
    /// Retrieves all feeds
    fn get_all_feeds(&self) -> Result<Vec<Feed>>;
    
    /// Retrieves a feed by its ID
    fn get_feed_by_id(&self, id: &FeedId) -> Result<Option<Feed>>;
    
    /// Retrieves a feed by its URL
    fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>>;
    
    /// Retrieves feeds by category ID
    fn get_feeds_by_category(&self, category_id: &CategoryId) -> Result<Vec<Feed>>;
    
    /// Saves a feed to the repository
    fn save_feed(&self, feed: &Feed) -> Result<()>;
    
    /// Updates an existing feed
    fn update_feed(&self, feed: &Feed) -> Result<()>;
    
    /// Deletes a feed by its ID
    fn delete_feed(&self, id: &FeedId) -> Result<()>;
    
    /// Searches for feeds by title or URL
    fn search_feeds(&self, query: &str) -> Result<Vec<Feed>>;
    
    /// Gets all feeds that need to be updated
    fn get_feeds_to_update(&self) -> Result<Vec<Feed>>;
    
    /// Gets feeds by date range
    fn get_feeds_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Feed>>;
}

/// Repository interface for category operations
pub trait CategoryRepository: Send + Sync {
    /// Retrieves all categories
    fn get_all_categories(&self) -> Result<Vec<Category>>;
    
    /// Retrieves a category by its ID
    fn get_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>>;
    
    /// Retrieves child categories by parent ID
    fn get_categories_by_parent(&self, parent_id: &CategoryId) -> Result<Vec<Category>>;
    
    /// Saves a category to the repository
    fn save_category(&self, category: &Category) -> Result<()>;
    
    /// Updates an existing category
    fn update_category(&self, category: &Category) -> Result<()>;
    
    /// Deletes a category by its ID
    fn delete_category(&self, id: &CategoryId) -> Result<()>;
    
    /// Searches for categories by name
    fn search_categories(&self, query: &str) -> Result<Vec<Category>>;
    
    /// Gets all root categories (categories without parents)
    fn get_root_categories(&self) -> Result<Vec<Category>>;
    
    /// Gets all child categories of a parent category
    fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>>;
    
    /// Gets categories by date range
    fn get_categories_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Category>>;
    
    /// Gets recently updated categories
    fn get_recently_updated_categories(&self, limit: usize) -> Result<Vec<Category>>;
    
    /// Gets the full category hierarchy
    fn get_category_hierarchy(&self) -> Result<Vec<Category>>;
}

/// Repository interface for feed-category relationship
pub trait FeedCategoryRepository: Send + Sync {
    /// Associates a feed with a category
    fn associate_feed_with_category(&self, feed_id: &FeedId, category_id: &CategoryId) -> Result<()>;
    
    /// Removes a feed from a category
    fn remove_feed_from_category(&self, feed_id: &FeedId, category_id: &CategoryId) -> Result<()>;
    
    /// Gets all categories for a feed
    fn get_categories_for_feed(&self, feed_id: &FeedId) -> Result<Vec<Category>>;
    
    /// Gets all feeds for a category
    fn get_feeds_for_category(&self, category_id: &CategoryId) -> Result<Vec<Feed>>;
}

/// Repository interface for article operations
pub trait ArticleRepository: Send + Sync {
    /// Creates a new article
    fn create_article(&self, article: &Article) -> Result<ArticleId>;

    /// Retrieves all articles
    fn get_all_articles(&self) -> Result<Vec<Article>>;
    
    /// Retrieves an article by its ID
    fn get_article(&self, id: &ArticleId) -> Result<Option<Article>>;
    
    /// Retrieves an article by URL
    fn get_article_by_url(&self, url: &str) -> Result<Option<Article>>;
    
    /// Retrieves articles by feed ID
    fn get_articles_by_feed(&self, feed_id: &FeedId) -> Result<Vec<Article>>;
    
    /// Retrieves articles by category ID
    fn get_articles_by_category(&self, category_id: &CategoryId) -> Result<Vec<Article>>;
    
    /// Retrieves unread articles
    fn get_unread_articles(&self) -> Result<Vec<Article>>;
    
    /// Retrieves favorite articles
    fn get_favorite_articles(&self) -> Result<Vec<Article>>;
    
    /// Retrieves articles by date
    fn get_articles_by_date(&self, date: DateTime<Utc>) -> Result<Vec<Article>>;
    
    /// Updates an existing article
    fn update_article(&self, article: &Article) -> Result<()>;
    
    /// Deletes an article by its ID
    fn delete_article(&self, id: &ArticleId) -> Result<()>;
    
    /// Searches for articles by content
    fn search_articles(&self, query: &str) -> Result<Vec<Article>>;
    
    /// Gets tags for an article
    fn get_article_tags(&self, article_id: ArticleId) -> Result<Vec<String>>;
    
    /// Adds tags to an article
    fn add_tags(&self, article_id: ArticleId, tags: &[String]) -> Result<()>;
    
    /// Adds a single tag to an article
    fn add_tag(&self, article_id: ArticleId, tag_id: &str) -> Result<()>;
    
    /// Removes a tag from an article
    fn remove_tag(&self, article_id: ArticleId, tag_id: &str) -> Result<()>;
    
    /// Removes articles older than the specified number of days
    fn cleanup_old_articles(&self, retention_days: i64) -> Result<usize>;
    
    /// Gets articles by date range
    fn get_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>>;
    
    /// Gets articles by tag
    fn get_articles_by_tag(&self, tag: &Tag) -> Result<Vec<Article>>;
    
    /// Marks an article as read
    fn mark_as_read(&self, article_id: &ArticleId) -> Result<()>;
    
    /// Adds an article to favorites
    fn add_to_favorites(&self, article_id: &ArticleId) -> Result<()>;
    
    /// Removes an article from favorites
    fn remove_from_favorites(&self, article_id: &ArticleId) -> Result<()>;
    
    /// Checks if an article is in favorites
    fn is_favorite(&self, article_id: &ArticleId) -> Result<bool>;
}

/// Repository interface for tag operations
pub trait TagRepository: Send + Sync {
    /// Retrieves all tags
    fn get_all_tags(&self) -> Result<Vec<Tag>>;
    
    /// Retrieves a tag by its ID
    fn get_tag_by_id(&self, id: &str) -> Result<Option<Tag>>;
    
    /// Retrieves tags by name (partial match)
    fn get_tags_by_name(&self, name: &str) -> Result<Vec<Tag>>;
    
    /// Saves a tag to the repository
    fn save_tag(&self, tag: &Tag) -> Result<()>;
    
    /// Updates an existing tag
    fn update_tag(&self, tag: &Tag) -> Result<()>;
    
    /// Deletes a tag by its ID
    fn delete_tag(&self, id: &str) -> Result<()>;
    
    /// Searches for tags by name
    fn search_tags(&self, query: &str) -> Result<Vec<Tag>>;
    
    /// Gets a tag by its name
    fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>>;
    
    /// Gets tags by date range
    fn get_tags_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Tag>>;
    
    /// Gets all articles with a specific tag
    fn get_articles_with_tag(&self, tag_id: &str) -> Result<Vec<Article>>;
    
    /// Gets the most used tags
    fn get_most_used_tags(&self, limit: usize) -> Result<Vec<(Tag, i64)>>;
}