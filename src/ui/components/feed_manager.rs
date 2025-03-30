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
use crate::base::repository::FeedRepository;

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
                                if let Err(e) = self.add_feed() {
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
        self.description = feed.description.clone().unwrap_or_default();
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
    fn fetch_feed_info(&mut self) -> Result<()> {
        if self.url_input.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        let url = Url::parse(&self.url_input)?;
        let feed_info = self.rss_service.fetch_feed_info(&url)?;
        
        self.title = feed_info.title;
        self.description = feed_info.description.unwrap_or_default();
        self.url = url.to_string();
        
        Ok(())
    }
    
    /// Saves the current feed
    fn save_feed(&mut self) -> Result<()> {
        if self.url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        let url = Url::parse(&self.url)?;
        let feed = Feed {
            id: FeedId::new(),
            url: url.to_string(),
            title: self.title.clone(),
            description: Some(self.description.clone()),
            category_id: self.selected_category.clone(),
            status: FeedStatus::Active,
            last_sync: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.rss_service.add_feed(&feed)?;
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        self.feeds = self.rss_service.get_all_feeds()?;
        self.categories = self.rss_service.get_all_categories()?;
        self.feeds.sort_by(|a, b| a.title.cmp(&b.title));
        self.categories.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(())
    }

    fn delete_feed(&mut self, feed_id: &FeedId) -> Result<()> {
        if let Err(e) = self.rss_service.delete_feed(feed_id) {
            error!("Failed to delete feed: {}", e);
            return Err(e);
        }
        self.feeds.retain(|f| f.id != *feed_id);
        Ok(())
    }

    fn add_feed(&mut self) -> Result<()> {
        if self.url_input.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        // First fetch feed info to validate URL and get feed details
        self.fetch_feed_info()?;
        
        // Then save the feed
        self.save_feed()?;
        
        // Refresh the feeds list
        self.refresh()?;
        
        Ok(())
    }

    fn sync_feed(&mut self, feed: &Feed) -> Result<()> {
        if let Err(e) = self.rss_service.update_feed(feed) {
            error!("Failed to sync feed: {}", e);
            return Err(e);
        }
        
        // Refresh the feeds list to update status
        self.refresh()?;
        Ok(())
    }
}