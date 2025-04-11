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
use crate::base::repository::{ArticleRepository, FeedRepository, CategoryRepository, TagRepository};
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
        let feeds = self.feed_repository.get_all_feeds().await?;
        for feed in feeds {
            if let Err(e) = self.fetch_feed(feed.url.as_str()).await {
                log::error!("Failed to fetch feed {}: {}", feed.title, e);
            }
        }
        Ok(())
    }

    pub async fn fetch_feed_by_id(&self, feed_id: &FeedId) -> Result<Feed> {
        if let Some(feed) = self.feed_repository.get_feed_by_id(feed_id).await? {
            self.fetch_feed(feed.url.as_str()).await
        } else {
            Err(anyhow::anyhow!("Feed not found"))
        }
    }

    /// Fetches and parses a feed
    pub async fn fetch_feed(&self, url: &str) -> Result<Feed> {
        let response = reqwest::get(url).await?;
        let content = response.bytes().await?;
        let feed_rs = feed_rs::parser::parse(&content[..])?;
        
        let mut feed = Feed::new(
            feed_rs.title.map(|t| t.content).unwrap_or_else(|| "Untitled Feed".to_string()),
            Url::parse(url)?
        );

        if let Some(desc) = feed_rs.description.map(|d| d.content) {
            feed = feed.with_description(desc);
        }

        if let Some(lang) = feed_rs.language {
            feed = feed.with_language(lang);
        }

        if let Some(link) = feed_rs.links.first() {
            if let Ok(site_url) = Url::parse(&link.href) {
                feed = feed.with_site_url(site_url);
            }
        }

        Ok(feed)
    }

    pub async fn fetch_articles(&self, feed: &Feed) -> Result<Vec<Article>> {
        let response = reqwest::get(feed.url.as_str()).await?;
        let content = response.bytes().await?;
        let feed_rs = feed_rs::parser::parse(&content[..])?;
        
        let mut articles = Vec::new();
        for entry in feed_rs.entries {
            let content = entry.content.and_then(|c| c.body);
            let summary = entry.summary.map(|s| s.content);
            
            let mut article = Article::new(
                feed.id.clone(),
                entry.title.map(|t| t.content).unwrap_or_else(|| "Untitled".to_string()),
                Url::parse(&entry.links.get(0).map(|l| l.href.clone()).unwrap_or_default())?
            );

            if let Some(content) = content {
                article = article.with_content(content);
            }

            if let Some(summary) = summary {
                article = article.with_summary(summary);
            }
            
            articles.push(article);
        }
        
        Ok(articles)
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

        self.feed_repository.save_feed(&feed).await?;
        Ok(())
    }

    /// Updates an existing feed
    pub async fn update_feed(&self, feed: &Feed) -> Result<()> {
        self.feed_repository.save_feed(feed).await?;
        Ok(())
    }

    /// Fetches new articles for all feeds that need to be updated
    pub async fn sync_all(&self) -> Result<()> {
        let feeds = self.feed_repository.get_feeds_to_update().await?;
        
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
                    self.feed_repository.update_feed(&failed_feed).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Gets all feeds in the repository
    pub async fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        Ok(self.feed_repository.get_all_feeds().await?)
    }

    /// Gets all feeds in a category
    pub async fn get_feeds_by_category(&self, category_id: &Option<CategoryId>) -> Result<Vec<Feed>> {
        match category_id {
            Some(id) => Ok(self.feed_repository.get_feeds_by_category(id).await?),
            None => Ok(self.feed_repository.get_all_feeds().await?)
        }
    }

    /// Gets all categories in the repository
    pub async fn get_all_categories(&self) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_all_categories().await?)
    }

    /// Gets all articles in a feed
    pub async fn get_articles_by_feed(&self, feed_id: &FeedId) -> Result<Vec<Article>> {
        Ok(self.article_repository.get_articles_by_feed(feed_id).await?)
    }

    /// Gets all articles in a category
    pub async fn fetch_articles_by_category(&self, category_id: &CategoryId) -> Result<Vec<Article>> {
        self.article_repository.get_articles_by_category(category_id).await
    }

    pub async fn get_unread_articles(&self) -> Result<Vec<Article>> {
        Ok(self.article_repository.get_unread_articles().await?)
    }

    pub async fn get_favorite_articles(&self) -> Result<Vec<Article>> {
        Ok(self.article_repository.get_favorite_articles().await?)
    }

    pub async fn search_articles(&self, query: &str) -> Result<Vec<Article>> {
        self.article_repository.search_articles(query).await
    }

    pub async fn fetch_articles_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Article>> {
        self.article_repository.get_articles_by_date_range(start, end).await
    }

    pub async fn delete_feed(&self, feed_id: &FeedId) -> Result<()> {
        self.feed_repository.delete_feed(feed_id).await?;
        Ok(())
    }

    pub async fn get_feed_by_id(&self, feed_id: &FeedId) -> Result<Option<Feed>> {
        Ok(self.feed_repository.get_feed_by_id(feed_id).await?)
    }

    pub async fn get_article(&self, article_id: &ArticleId) -> Result<Option<Article>> {
        self.article_repository.get_article(article_id).await
    }

    pub async fn get_all_articles(&self) -> Result<Vec<Article>> {
        self.article_repository.get_all_articles().await
    }

    pub async fn get_category_by_id(&self, category_id: &CategoryId) -> Result<Option<Category>> {
        Ok(self.category_repository.get_category_by_id(category_id).await?)
    }

    pub async fn get_categories_by_parent(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        // Wrap the CategoryId in an Option since the repository method expects Option<CategoryId>
        let parent_option = Some(parent_id.clone());
        Ok(self.category_repository.get_categories_by_parent(&parent_option).await?)
    }

    pub async fn get_root_categories(&self) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_root_categories().await?)
    }

    pub async fn get_child_categories(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_child_categories(parent_id).await?)
    }

    pub async fn search_categories(&self, name: &str) -> Result<Vec<Category>> {
        Ok(self.category_repository.search_categories(name).await?)
    }

    pub async fn get_recently_updated_categories(&self, limit: usize) -> Result<Vec<Category>> {
        Ok(self.category_repository.get_recently_updated_categories(limit).await?)
    }

    pub async fn save_category(&self, category: &Category) -> Result<()> {
        Ok(self.category_repository.save_category(category).await?)
    }

    pub async fn update_category(&self, category: &Category) -> Result<()> {
        Ok(self.category_repository.update_category(category).await?)
    }

    pub async fn delete_category(&self, category_id: &CategoryId) -> Result<()> {
        Ok(self.category_repository.delete_category(category_id).await?)
    }

    /// Gets articles by tag
    pub async fn get_articles_by_tag(&self, tag_id: &TagId) -> Result<Vec<Article>> {
        // Get the tag ID as a string
        let tag_id_str = tag_id.to_string();
        
        // Option 1: Try to get the tag by name (using the tag ID string)
        // This might work if TagId.to_string() returns the tag name or if tags can be retrieved by ID string
        match self.tag_repository.get_tag_by_name(&tag_id_str).await {
            Ok(Some(tag)) => {
                // If we found the tag, use its name to get articles
                return self.article_repository.get_articles_by_tag(&tag.name).await;
            },
            _ => {
                // Option 2: Use the articles_with_tag method that returns article IDs
                let article_ids = self.tag_repository.get_articles_with_tag(tag_id).await?;
                
                // Fetch each article by ID
                let mut articles = Vec::new();
                for id_str in article_ids {
                    let article_id = ArticleId(id_str);
                    if let Ok(Some(article)) = self.article_repository.get_article(&article_id).await {
                        articles.push(article);
                    }
                }
                
                Ok(articles)
            }
        }
    }

    /// Syncs a feed
    pub async fn sync_feed(&self, feed_id: &FeedId) -> Result<()> {
        self.fetch_feed_by_id(feed_id).await?;
        Ok(())
    }

    /// Syncs all feeds
    pub async fn sync_all_feeds(&self) -> Result<()> {
        let feeds = self.feed_repository.get_feeds_to_update().await?;
        for feed in feeds {
            if let Err(e) = self.sync_feed(&feed.id).await {
                log::error!("Failed to sync feed {}: {}", feed.id, e);
            }
        }
        Ok(())
    }

    /// Updates an article
    pub async fn update_article(&self, article: &Article) -> Result<()> {
        self.article_repository.update_article(article).await
    }

    /// Gets a feed by its URL
    pub async fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>> {
        Ok(self.feed_repository.get_feed_by_url(url).await?)
    }
}