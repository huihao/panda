use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::models::category::{Category, CategoryId};
use crate::models::feed::{Feed, FeedId};
use crate::models::tag::{Tag, TagId};

// ==================== CategoryRepository ====================
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    /// 根据ID获取分类
    async fn get_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>>;
    
    /// 获取所有分类
    async fn get_all_categories(&self) -> Result<Vec<Category>>;
    
    /// 根据父ID获取分类
    async fn get_categories_by_parent(&self, parent_id: &Option<CategoryId>) -> Result<Vec<Category>>;
    
    /// 获取所有根分类（没有父分类的分类）
    async fn get_root_categories(&self) -> Result<Vec<Category>>;
    
    /// 获取某个分类的所有子分类
    async fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>>;
    
    /// 保存分类
    async fn save_category(&self, category: &Category) -> Result<()>;
    
    /// 更新分类
    async fn update_category(&self, category: &Category) -> Result<()>;
    
    /// 删除分类
    async fn delete_category(&self, id: &CategoryId) -> Result<()>;
    
    /// 搜索分类
    async fn search_categories(&self, name: &str) -> Result<Vec<Category>>;
    
    /// 获取最近更新的分类
    async fn get_recently_updated_categories(&self, limit: usize) -> Result<Vec<Category>>;
    
    /// 根据日期范围获取分类
    async fn get_categories_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Category>>;
    
    /// 获取分类层次结构
    async fn get_category_hierarchy(&self) -> Result<Vec<Category>>;
}

// ==================== ArticleRepository ====================
#[async_trait]
pub trait ArticleRepository: Send + Sync {
    /// 保存文章
    async fn save_article(&self, article: &Article) -> Result<()>;
    
    /// 根据ID获取文章
    async fn get_article(&self, id: &ArticleId) -> Result<Option<Article>>;
    
    /// 根据URL获取文章
    async fn get_article_by_url(&self, url: &str) -> Result<Option<Article>>;
    
    /// 获取所有文章
    async fn get_all_articles(&self) -> Result<Vec<Article>>;
    
    /// 获取某个Feed的所有文章
    async fn get_articles_by_feed(&self, feed_id: &FeedId) -> Result<Vec<Article>>;
    
    /// 获取某个分类的所有文章
    async fn get_articles_by_category(&self, category_id: &CategoryId) -> Result<Vec<Article>>;
    
    /// 获取所有未读文章
    async fn get_unread_articles(&self) -> Result<Vec<Article>>;
    
    /// 获取所有收藏文章
    async fn get_favorite_articles(&self) -> Result<Vec<Article>>;
    
    /// 更新文章
    async fn update_article(&self, article: &Article) -> Result<()>;
    
    /// 删除文章
    async fn delete_article(&self, id: &ArticleId) -> Result<()>;
    
    /// 添加标签到文章
    async fn add_tag(&self, article_id: &ArticleId, tag: &str) -> Result<()>;
    
    /// 从文章移除标签
    async fn remove_tag(&self, article_id: &ArticleId, tag: &str) -> Result<()>;
    
    /// 获取文章的所有标签
    async fn get_article_tags(&self, article_id: &ArticleId) -> Result<Vec<String>>;
    
    /// 获取带有特定标签的所有文章
    async fn get_articles_by_tag(&self, tag: &str) -> Result<Vec<Article>>;
    
    /// 根据日期范围获取文章
    async fn get_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>>;
    
    /// 搜索文章
    async fn search_articles(&self, query: &str) -> Result<Vec<Article>>;
}

// ==================== FeedRepository ====================
#[async_trait]
pub trait FeedRepository: Send + Sync {
    /// 保存Feed
    async fn save_feed(&self, feed: &Feed) -> Result<()>;
    
    /// 根据ID获取Feed
    async fn get_feed_by_id(&self, id: &FeedId) -> Result<Option<Feed>>;
    
    /// 根据URL获取Feed
    async fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>>;
    
    /// 获取所有Feed
    async fn get_all_feeds(&self) -> Result<Vec<Feed>>;
    
    /// 获取某个分类的所有Feed
    async fn get_feeds_by_category(&self, category_id: &CategoryId) -> Result<Vec<Feed>>;
    
    /// 获取所有启用的Feed
    async fn get_enabled_feeds(&self) -> Result<Vec<Feed>>;
    
    /// 获取需要更新的Feed
    async fn get_feeds_to_update(&self) -> Result<Vec<Feed>>;
    
    /// 更新Feed
    async fn update_feed(&self, feed: &Feed) -> Result<()>;
    
    /// 删除Feed
    async fn delete_feed(&self, id: &FeedId) -> Result<()>;
    
    /// 搜索Feed
    async fn search_feeds(&self, query: &str) -> Result<Vec<Feed>>;
    
    /// 根据日期范围获取Feed
    async fn get_feeds_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Feed>>;
    
    /// 获取最近更新的Feed
    async fn get_recently_updated_feeds(&self, limit: usize) -> Result<Vec<Feed>>;
    
    /// 获取最活跃的Feed
    async fn get_most_active_feeds(&self, limit: usize) -> Result<Vec<Feed>>;
}

// ==================== TagRepository ====================
#[async_trait]
pub trait TagRepository: Send + Sync {
    /// 保存标签
    async fn save_tag(&self, tag: &Tag) -> Result<()>;
    
    /// 根据ID获取标签
    async fn get_tag_by_id(&self, id: &TagId) -> Result<Option<Tag>>;
    
    /// 根据名称获取标签
    async fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>>;
    
    /// 获取所有标签
    async fn get_all_tags(&self) -> Result<Vec<Tag>>;
    
    /// 更新标签
    async fn update_tag(&self, tag: &Tag) -> Result<()>;
    
    /// 删除标签
    async fn delete_tag(&self, id: &TagId) -> Result<()>;
    
    /// 搜索标签
    async fn search_tags(&self, query: &str) -> Result<Vec<Tag>>;
    
    /// 根据日期范围获取标签
    async fn get_tags_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Tag>>;
    
    /// 获取文章的标签
    async fn get_article_tags(&self, article_id: &ArticleId) -> Result<Vec<Tag>>;
    
    /// 将标签添加到文章
    async fn add_tag_to_article(&self, article_id: &ArticleId, tag_id: &TagId) -> Result<()>;
    
    /// 从文章移除标签
    async fn remove_tag_from_article(&self, article_id: &ArticleId, tag_id: &TagId) -> Result<()>;
    
    /// 获取带有特定标签的所有文章ID
    async fn get_articles_with_tag(&self, tag_id: &TagId) -> Result<Vec<String>>;
    
    /// 获取使用最多的标签
    async fn get_most_used_tags(&self, limit: usize) -> Result<Vec<Tag>>;
}