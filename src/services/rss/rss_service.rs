use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use feed_rs::parser;
use log::{error, info};
use url::Url;
use uuid::Uuid;
use std::sync::Arc;
use reqwest::Client;
use feed_rs::model::Feed as FeedRs;

use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::models::feed::{Feed, FeedId, FeedStatus};
use crate::data::{ArticleRepository, FeedRepository};
use crate::base::repository::{CategoryRepository, TagRepository};
use crate::models::category::{Category, CategoryId};
use crate::models::tag::{Tag, TagId};

/// Service for managing RSS feeds
pub struct RssService {
    article_repository: Arc<dyn ArticleRepository>,
    feed_repository: Arc<dyn FeedRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    tag_repository: Arc<dyn TagRepository>,
    client: Client,
}

impl RssService {
    /// Creates a new RSS service
    pub fn new(
        article_repository: Arc<dyn ArticleRepository>,
        feed_repository: Arc<dyn FeedRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        tag_repository: Arc<dyn TagRepository>,
    ) -> Self {
        Self {
            article_repository,
            feed_repository,
            category_repository,
            tag_repository,
            client: Client::new(),
        }
    }

    /// Fetches all feeds
    pub async fn fetch_all_feeds(&self) -> Result<()> {
        let feeds = self.feed_repository.get_all_feeds()?;
        for feed in feeds {
            if let Err(e) = self.fetch_feed(&feed.id).await {
                log::error!("Failed to fetch feed {}: {}", feed.title, e);
            }
        }
        Ok(())
    }

    /// Fetches and parses a feed
    pub async fn fetch_feed(&self, feed_id: &FeedId) -> Result<Feed> {
        let feed = self.feed_repository.get_feed_by_id(feed_id)?
            .ok_or_else(|| anyhow::anyhow!("Feed not found"))?;

        let response = reqwest::get(feed.url.as_str()).await?;
        let content = response.text().await?;
        let feed_data = parser::parse(content.as_bytes())?;

        let mut article = Article::new(
            feed.id.clone(),
            feed_data.title.map(|t| t.content).unwrap_or_default(),
            feed.url.clone(),
        );

        if let Some(author) = feed_data.authors.first() {
            article = article.with_author(author.name.clone());
        }

        if let Some(entry) = feed_data.entries.first() {
            if let Some(content) = entry.content.first() {
                article = article.with_content(content.body.clone());
            }
            if let Some(summary) = &entry.summary {
                article = article.with_summary(summary.content.clone());
            }
            if let Some(published) = entry.published {
                article = article.with_published_at(published);
            }
        }

        self.article_repository.create_article(&article)?;
        
        let mut updated_feed = feed.clone();
        updated_feed.update_fetch_times(Utc::now(), Utc::now() + chrono::Duration::hours(1));
        self.feed_repository.update_feed(&updated_feed)?;

        Ok(updated_feed)
    }

    /// Adds a new feed
    pub async fn add_feed(&self, url: &str) -> Result<()> {
        let url = Url::parse(url)?;
        let response = reqwest::get(url.as_str()).await?;
        let content = response.text().await?;
        let feed_data = parser::parse(content.as_bytes())?;

        let feed = Feed::new(
            feed_data.title.map(|t| t.content).unwrap_or_default(),
            url,
        );

        self.feed_repository.save_feed(&feed)?;
        Ok(())
    }

    /// Updates an existing feed
    pub async fn update_feed(&self, feed: &Feed) -> Result<()> {
        self.feed_repository.save_feed(feed)?;
        Ok(())
    }

    /// Fetches new articles for all feeds that need to be updated
    pub async fn sync_all(&self) -> Result<()> {
        let feeds = self.feed_repository.get_feeds_to_update()?;
        
        for feed in feeds {
            match self.update_feed(&feed).await {
                Ok(_) => {
                    info!("Successfully updated feed: {}", feed.title);
                }
                Err(e) => {
                    error!("Failed to update feed {}: {}", feed.title, e);
                    let mut failed_feed = feed.clone();
                    failed_feed.update_status(FeedStatus::Error);
                    failed_feed.update_error_message(e.to_string());
                    failed_feed.update_fetch_times(Utc::now(), Utc::now() + chrono::Duration::hours(1));
                    self.feed_repository.update_feed(&failed_feed)?;
                }
            }
        }
        
        Ok(())
    }

    /// Gets all feeds in the repository
    pub async fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        Ok(self.feed_repository.get_all_feeds()?)
    }

    /// Gets all feeds in a category
    pub async fn get_feeds_by_category(&self, category_id: &CategoryId) -> Result<Vec<Feed>> {
        Ok(self.feed_repository.get_feeds_by_category(category_id)?)
    }

    /// Gets all categories in the repository
    pub async fn get_all_categories(&self) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_all_categories()?)
    }

    /// Gets all articles in a feed
    pub async fn get_articles_by_feed(&self, feed_id: &FeedId) -> Result<Vec<Article>> {
        Ok(self.article_repository.get_articles_by_feed(feed_id)?)
    }

    /// Gets all articles in a category
    pub async fn fetch_articles_by_category(&self, category_id: &CategoryId) -> Result<Vec<Article>> {
        self.article_repository.get_articles_by_category(category_id)
    }

    /// Gets all unread articles
    pub async fn get_unread_articles(&self) -> Result<Vec<Article>> {
        Ok(self.article_repository.get_unread_articles()?)
    }

    /// Gets all favorited articles
    pub async fn get_favorite_articles(&self) -> Result<Vec<Article>> {
        Ok(self.article_repository.get_favorite_articles()?)
    }

    /// Searches for articles by title or content
    pub async fn search_articles(&self, query: &str) -> Result<Vec<Article>> {
        self.article_repository.search_articles(query)
    }

    /// Gets articles by date range
    pub async fn fetch_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>> {
        self.article_repository.get_articles_by_date_range(start, end)
    }

    /// Deletes a feed and all its articles
    pub async fn delete_feed(&self, feed_id: &FeedId) -> Result<()> {
        self.feed_repository.delete_feed(feed_id)?;
        Ok(())
    }

    pub async fn get_feed_by_id(&self, feed_id: &FeedId) -> Result<Option<Feed>> {
        Ok(self.feed_repository.get_feed_by_id(feed_id)?)
    }

    /// Gets an article by ID
    pub async fn get_article(&self, article_id: &ArticleId) -> Result<Option<Article>> {
        self.article_repository.get_article(article_id)
    }

    /// Gets all articles
    pub async fn get_all_articles(&self) -> Result<Vec<Article>> {
        self.article_repository.get_all_articles()
    }

    pub async fn get_category_by_id(&self, category_id: &CategoryId) -> Result<Option<Category>> {
        Ok(self.category_repository.get_category_by_id(category_id)?)
    }

    pub async fn get_categories_by_parent(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_categories_by_parent(parent_id)?)
    }

    pub async fn get_root_categories(&self) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_root_categories()?)
    }

    pub async fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_child_categories(parent_id)?)
    }

    pub async fn search_categories(&self, name: &str) -> Result<Vec<Category>> {
        Ok(self.category_repository.search_categories(name)?)
    }

    pub async fn get_recently_updated_categories(&self, limit: usize) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_recently_updated_categories(limit)?)
    }

    pub async fn save_category(&self, category: &Category) -> Result<()> {
        Ok(self.category_repository.save_category(category)?)
    }

    pub async fn update_category(&self, category: &Category) -> Result<()> {
        Ok(self.category_repository.update_category(category)?)
    }

    pub async fn delete_category(&self, category_id: &CategoryId) -> Result<()> {
        Ok(self.category_repository.delete_category(category_id)?)
    }

    /// Gets articles by tag
    pub async fn get_articles_by_tag(&self, tag_id: &TagId) -> Result<Vec<Article>> {
        self.article_repository.get_articles_by_tag(tag_id)
    }

    /// Syncs a feed
    pub async fn sync_feed(&self, feed_id: &FeedId) -> Result<()> {
        self.fetch_feed(feed_id).await?;
        Ok(())
    }

    /// Syncs all feeds
    pub async fn sync_all_feeds(&self) -> Result<()> {
        let feeds = self.feed_repository.get_feeds_to_fetch()?;
        for feed in feeds {
            if let Err(e) = self.sync_feed(&feed.id).await {
                log::error!("Failed to sync feed {}: {}", feed.id, e);
            }
        }
        Ok(())
    }
}