use egui::{Ui, Window, TextEdit, ComboBox, Button, RichText, Color32};
use log::{info, error};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use url::Url;
use std::sync::Arc;

use crate::models::feed::{Feed, FeedId, FeedStatus};
use crate::models::category::{Category, CategoryId};
use crate::services::rss::RssService;
use crate::ui::styles::{AppColors, DEFAULT_PADDING};
use crate::base::repository_traits::FeedRepository;

/// Feed management dialog component
pub struct FeedManager {
    pub visible: bool,
    pub url: String,
    pub title: String,
    pub description: String,
    pub selected_category: Option<CategoryId>,
    pub categories: Vec<Category>,
    pub feeds: Vec<Feed>,
    pub colors: AppColors,
    pub rss_service: Arc<RssService>,
    url_input: String,
    error_message: Option<String>,
}

impl FeedManager {
    /// Creates a new feed manager
    pub fn new(rss_service: Arc<RssService>, colors: AppColors) -> Self {
        Self {
            visible: false,
            url: String::new(),
            title: String::new(),
            description: String::new(),
            selected_category: None,
            categories: Vec::new(),
            feeds: Vec::new(),
            colors,
            rss_service,
            url_input: String::new(),
            error_message: None,
        }
    }
    
    /// Shows the feed manager dialog
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        if self.visible {
            Window::new("Add Feed")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Enter Feed URL").color(self.colors.text));
                        ui.add(TextEdit::singleline(&mut self.url_input)
                            .hint_text("https://example.com/feed.xml")
                            .desired_width(300.0));
                        
                        if let Some(error) = &self.error_message {
                            ui.label(RichText::new(error).color(self.colors.error));
                        }
                        
                        ui.horizontal(|ui| {
                            if ui.button("Add").clicked() {
                                // 使用tokio::task::block_in_place在UI线程中调用异步方法
                                if let Err(e) = tokio::task::block_in_place(|| {
                                    let rt = tokio::runtime::Handle::current();
                                    rt.block_on(async {
                                        self.add_feed().await
                                    })
                                }) {
                                    error!("Failed to add feed: {}", e);
                                    self.error_message = Some(e.to_string());
                                } else {
                                    self.visible = false;
                                    self.url_input.clear();
                                    self.error_message = None;
                                }
                            }
                            
                            if ui.button("Cancel").clicked() {
                                self.visible = false;
                                self.url_input.clear();
                                self.error_message = None;
                            }
                        });
                    });
                });
        }
        Ok(())
    }
    
    /// Opens the dialog in add mode
    pub fn open_add(&mut self) {
        self.visible = true;
        self.url.clear();
        self.title.clear();
        self.description.clear();
        self.selected_category = None;
    }
    
    /// Opens the dialog in edit mode
    pub fn open_edit(&mut self, feed: Feed) {
        self.visible = true;
        self.url = feed.url.to_string();
        self.title = feed.title.clone();
        // Feed doesn't have a description field, so we'll just use an empty string
        self.description = String::new();
        self.selected_category = feed.category_id.clone();
    }
    
    /// Closes the dialog
    pub fn close(&mut self) {
        self.visible = false;
        self.url.clear();
        self.title.clear();
        self.description.clear();
        self.selected_category = None;
    }
    
    /// Returns whether the dialog is currently open
    pub fn is_open(&self) -> bool {
        self.visible
    }
    
    /// Fetches feed information from the URL
    async fn fetch_feed_info(&mut self) -> Result<()> {
        if self.url_input.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        let url = Url::parse(&self.url_input)?;
        // Pass the URL as a string since fetch_feed expects &str
        let feed = self.rss_service.fetch_feed(url.as_str()).await?;
        
        self.title = feed.title;
        // Feed doesn't have a description field
        self.description = String::new();
        self.url = url.to_string();
        
        Ok(())
    }
    
    /// Saves the current feed
    async fn save_feed(&mut self) -> Result<()> {
        if self.url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        let url = Url::parse(&self.url)?;
        let feed = Feed::new(
            self.title.clone(),
            url,
        );

        self.rss_service.add_feed(&feed.url.to_string()).await?;
        Ok(())
    }

    async fn refresh(&mut self) -> Result<()> {
        self.feeds = self.rss_service.get_all_feeds().await?;
        self.categories = self.rss_service.get_all_categories().await?;
        self.feeds.sort_by(|a, b| a.title.cmp(&b.title));
        self.categories.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(())
    }

    async fn delete_feed(&mut self, feed_id: &FeedId) -> Result<()> {
        if let Err(e) = self.rss_service.delete_feed(feed_id).await {
            error!("Failed to delete feed: {}", e);
            return Err(e);
        }
        self.feeds.retain(|f| f.id != *feed_id);
        Ok(())
    }

    async fn add_feed(&mut self) -> Result<()> {
        if self.url_input.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        // First fetch feed info to validate URL and get feed details
        self.fetch_feed_info().await?;
        
        // Then save the feed
        self.save_feed().await?;
        
        // Refresh the feeds list
        self.refresh().await?;
        
        Ok(())
    }

    async fn sync_feed(&mut self, feed: &Feed) -> Result<()> {
        if let Err(e) = self.rss_service.update_feed(feed).await {
            error!("Failed to sync feed: {}", e);
            return Err(e);
        }
        
        // Refresh the feeds list to update status
        self.refresh().await?;
        Ok(())
    }
}