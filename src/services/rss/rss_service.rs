use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use feed_rs::parser;
use url::Url;
use uuid::Uuid;
use std::sync::Arc;

use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::models::feed::{Feed, FeedId, FeedStatus};
use crate::data::{ArticleRepository, FeedRepository};

pub struct RssService {
    feed_repository: Arc<dyn FeedRepository>,
    article_repository: Arc<dyn ArticleRepository>,
}

impl RssService {
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        article_repository: Arc<dyn ArticleRepository>,
    ) -> Self {
        Self {
            feed_repository,
            article_repository,
        }
    }

    /// Fetches all feeds and updates articles database
    /// 
    /// This method retrieves all registered feeds from the repository
    /// and fetches new articles from each one, storing them in the database.
    /// If fetching a particular feed fails, it will log the error and continue
    /// with the next feed to ensure maximum resilience.
    pub async fn fetch_all_feeds(&self) -> Result<()> {
        let feeds = self.feed_repository.get_all_feeds()?;
        let mut success_count = 0;
        let mut error_count = 0;
        
        for feed in feeds {
            match self.update_feed(&feed.id).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    error_count += 1;
                    log::error!("Failed to update feed {}: {}", feed.url, e);
                    // Continue with next feed instead of aborting
                }
            }
        }
        
        log::info!("Feed update complete: {} successful, {} failed", success_count, error_count);
        
        if error_count > 0 && success_count == 0 {
            return Err(anyhow!("All feed updates failed"));
        }
        
        Ok(())
    }

    pub async fn fetch_feed(&self, feed: &Feed) -> Result<Vec<Article>> {
        let response = reqwest::get(feed.url.clone()).await?;
        let content = response.bytes().await?;
        let parsed = parser::parse(&content[..])?;

        let mut articles = Vec::new();
        for entry in parsed.entries {
            let id = ArticleId(Uuid::new_v4().to_string());
            
            let article = Article {
                id,
                feed_id: feed.id.0.clone(),
                title: entry.title.map(|t| t.content).unwrap_or_default(),
                url: entry.links.first()
                    .ok_or_else(|| anyhow!("Article has no URL"))?
                    .href.parse()?,
                author: entry.authors.first().map(|a| a.name.clone()),
                content: entry.content.and_then(|c| c.body).unwrap_or_default(),
                summary: entry.summary.map(|s| s.content),
                published_at: entry.published.or(entry.updated)
                    .ok_or_else(|| anyhow!("Article has no date"))?
                    .with_timezone(&Utc),
                read_status: false,
                is_favorite: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                thumbnail_url: entry.media.first()
                    .and_then(|m| m.thumbnails.first())
                    .and_then(|t| t.image.uri.parse().ok()),
                tags: Vec::new(),
            };

            articles.push(article);
        }

        Ok(articles)
    }

    pub async fn add_feed(&self, feed_url: Url) -> Result<FeedId> {
        // Check if feed already exists
        if let Some(existing_feed) = self.feed_repository.get_feed_by_url(feed_url.as_str())? {
            return Ok(existing_feed.id);
        }

        // Fetch and parse the feed
        let response = reqwest::get(feed_url.clone()).await?;
        let content = response.bytes().await?;
        let parsed = parser::parse(&content[..])?;

        let now = Utc::now();
        let feed = Feed {
            id: FeedId(Uuid::new_v4().to_string()),
            url: feed_url,
            title: parsed.title.map(|t| t.content).unwrap_or_else(|| "Untitled Feed".to_string()),
            description: parsed.description.map(|d| d.content),
            icon_url: parsed.icon.and_then(|i| i.uri.parse().ok()),
            category_id: None,
            status: FeedStatus::Active,
            error_message: None,
            last_fetch: Some(now),
            next_fetch: Some(now),
            update_interval: Some(3600), // Default to 1 hour
            created_at: now,
            updated_at: now,
        };

        self.feed_repository.save_feed(&feed)?;

        // Add initial articles
        for entry in parsed.entries {
            let url: Url = entry.links.first()
                .ok_or_else(|| anyhow!("Article has no URL"))?
                .href.parse()?;

            // Skip if article already exists
            if self.article_repository.get_article_by_url(url.as_str())?.is_some() {
                continue;
            }

            let article = Article {
                id: ArticleId(Uuid::new_v4().to_string()),
                feed_id: feed.id.0.clone(),
                title: entry.title.map(|t| t.content).unwrap_or_default(),
                url,
                author: entry.authors.first().map(|a| a.name.clone()),
                content: entry.content.and_then(|c| c.body).unwrap_or_default(),
                summary: entry.summary.map(|s| s.content),
                published_at: entry.published.or(entry.updated)
                    .ok_or_else(|| anyhow!("Article has no date"))?
                    .with_timezone(&Utc),
                read_status: false,
                is_favorite: false,
                created_at: now,
                updated_at: now,
                thumbnail_url: entry.media.first()
                    .and_then(|m| m.thumbnails.first())
                    .and_then(|t| t.image.uri.parse().ok()),
                tags: Vec::new(),
            };

            self.article_repository.create_article(&article)?;
        }

        Ok(feed.id)
    }

    pub async fn update_feed(&self, feed_id: &FeedId) -> Result<()> {
        let mut feed = self.feed_repository.get_feed_by_id(feed_id)?
            .ok_or_else(|| anyhow!("Feed not found"))?;

        let articles = self.fetch_feed(&feed).await?;
        for article in articles {
            if self.article_repository.get_article_by_url(article.url.as_str())?.is_none() {
                self.article_repository.create_article(&article)?;
            }
        }

        let now = Utc::now();
        feed.last_fetch = Some(now);
        feed.next_fetch = Some(now + chrono::Duration::hours(1));
        feed.updated_at = now;

        self.feed_repository.update_feed(&feed)?;
        Ok(())
    }
}