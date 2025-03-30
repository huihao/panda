use std::path::Path;
use anyhow::Result;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use crate::models::{feed::Feed, category::Category};
use std::sync::Arc;
use log::error;
use url::Url;
use opml::{Outline, OPML};
use chrono::Utc;

use crate::models::category::CategoryId;
use crate::models::feed::{Feed, FeedId};
use crate::services::rss::RssService;

/// Service for importing and exporting OPML files
pub struct OpmlService {
    rss_service: Arc<RssService>,
}

impl OpmlService {
    /// Creates a new OPML service
    pub fn new(rss_service: Arc<RssService>) -> Self {
        Self { rss_service }
    }

    /// Imports feeds from an OPML file
    pub async fn import_opml(&self, content: &str) -> Result<()> {
        let opml = opml::Opml::new(content)?;
        let mut category_stack: Vec<CategoryId> = Vec::new();

        for outline in opml.body.outlines {
            if let Some(url) = outline.xml_url.as_ref() {
                let url = Url::parse(url)?;
                let mut feed = Feed::new(outline.text.unwrap_or_default(), url);
                
                if let Some(category_name) = outline.text {
                    if let Ok(categories) = self.rss_service.search_categories(&category_name).await {
                        if let Some(category) = categories.first() {
                            feed = feed.with_category(category.id.clone());
                        }
                    }
                }
                
                self.rss_service.add_feed(&feed.url.to_string()).await?;
            } else if let Some(text) = outline.text {
                let mut category = Category::new(text.clone());
                if let Some(parent_id) = category_stack.last() {
                    category = category.with_parent_id(parent_id.clone());
                }
                self.rss_service.save_category(&category).await?;
                category_stack.push(category.id);
            }
        }

        Ok(())
    }

    /// Exports feeds to an OPML file
    pub async fn export_opml(&self) -> Result<String> {
        let feeds = self.rss_service.get_all_feeds().await?;
        let categories = self.rss_service.get_all_categories().await?;

        let mut opml = OPML::default();
        opml.head.title = Some("Panda RSS Feeds".to_string());

        for category in categories {
            let category_feeds: Vec<_> = feeds
                .iter()
                .filter(|f| f.category_id.as_ref() == Some(&category.id))
                .collect();

            if !category_feeds.is_empty() {
                let mut category_outline = opml::Outline::default();
                category_outline.text = Some(category.name);
                category_outline.title = Some(category.name);

                for feed in category_feeds {
                    let mut feed_outline = opml::Outline::default();
                    feed_outline.text = Some(feed.title.clone());
                    feed_outline.title = Some(feed.title.clone());
                    feed_outline.xml_url = Some(feed.url.to_string());
                    category_outline.outlines.push(feed_outline);
                }

                opml.body.outlines.push(category_outline);
            }
        }

        let uncategorized_feeds: Vec<_> = feeds
            .iter()
            .filter(|f| f.category_id.is_none())
            .collect();

        if !uncategorized_feeds.is_empty() {
            let mut uncategorized_outline = opml::Outline::default();
            uncategorized_outline.text = Some("Uncategorized".to_string());
            uncategorized_outline.title = Some("Uncategorized".to_string());

            for feed in uncategorized_feeds {
                let mut feed_outline = opml::Outline::default();
                feed_outline.text = Some(feed.title.clone());
                feed_outline.title = Some(feed.title.clone());
                feed_outline.xml_url = Some(feed.url.to_string());
                uncategorized_outline.outlines.push(feed_outline);
            }

            opml.body.outlines.push(uncategorized_outline);
        }

        Ok(opml.to_string())
    }
} 