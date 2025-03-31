use anyhow::Result;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

use crate::models::{
    article::{Article, ArticleId},
    feed::{Feed, FeedId},
    category::{Category, CategoryId},
    tag::{Tag, TagId},
};

#[async_trait]
pub trait ArticleRepository: Send + Sync {
    async fn get_article(&self, id: &ArticleId) -> Result<Option<Article>>;
    async fn get_all_articles(&self) -> Result<Vec<Article>>;
    async fn get_articles_by_feed(&self, feed_id: &FeedId) -> Result<Vec<Article>>;
    async fn get_articles_by_category(&self, category_id: &CategoryId) -> Result<Vec<Article>>;
    async fn get_articles_by_tag(&self, tag_id: &TagId) -> Result<Vec<Article>>;
    async fn get_unread_articles(&self) -> Result<Vec<Article>>;
    async fn get_favorite_articles(&self) -> Result<Vec<Article>>;
    async fn search_articles(&self, query: &str) -> Result<Vec<Article>>;
    async fn get_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>>;
    async fn save_article(&self, article: &Article) -> Result<()>;
    async fn update_article(&self, article: &Article) -> Result<()>;
    async fn delete_article(&self, id: &ArticleId) -> Result<()>;
}

#[async_trait]
pub trait FeedRepository: Send + Sync {
    async fn get_feed_by_id(&self, id: &FeedId) -> Result<Option<Feed>>;
    async fn get_all_feeds(&self) -> Result<Vec<Feed>>;
    async fn get_feeds_by_category(&self, category_id: &Option<CategoryId>) -> Result<Vec<Feed>>;
    async fn get_feeds_to_update(&self) -> Result<Vec<Feed>>;
    async fn save_feed(&self, feed: &Feed) -> Result<()>;
    async fn update_feed(&self, feed: &Feed) -> Result<()>;
    async fn delete_feed(&self, id: &FeedId) -> Result<()>;
}

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn get_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>>;
    async fn get_all_categories(&self) -> Result<Vec<Category>>;
    async fn get_categories_by_parent(&self, parent_id: &Option<CategoryId>) -> Result<Vec<Category>>;
    async fn get_root_categories(&self) -> Result<Vec<Category>>;
    async fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>>;
    async fn save_category(&self, category: &Category) -> Result<()>;
    async fn update_category(&self, category: &Category) -> Result<()>;
    async fn delete_category(&self, id: &CategoryId) -> Result<()>;
}

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn get_tag_by_id(&self, id: &TagId) -> Result<Option<Tag>>;
    async fn get_all_tags(&self) -> Result<Vec<Tag>>;
    async fn get_tags_by_article(&self, article_id: &ArticleId) -> Result<Vec<Tag>>;
    async fn save_tag(&self, tag: &Tag) -> Result<()>;
    async fn update_tag(&self, tag: &Tag) -> Result<()>;
    async fn delete_tag(&self, id: &TagId) -> Result<()>;
    async fn add_tag_to_article(&self, article_id: &ArticleId, tag_id: &TagId) -> Result<()>;
    async fn remove_tag_from_article(&self, article_id: &ArticleId, tag_id: &TagId) -> Result<()>;
}