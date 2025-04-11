use egui::{Ui, Window, TextEdit, ComboBox, Button, RichText, Color32, ScrollArea, Checkbox};
use log::{info, error};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use url::Url;
use std::sync::Arc;
use std::collections::HashSet;

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
    pub selected_categories: HashSet<CategoryId>, // Add support for multiple categories
    pub categories: Vec<Category>,
    pub feeds: Vec<Feed>,
    pub colors: AppColors,
    pub rss_service: Arc<RssService>,
    url_input: String,
    error_message: Option<String>,
    show_preview: bool,
    is_fetching: bool,
    feed_preview: Option<Feed>,
    is_saving: bool,
}

impl FeedManager {
    /// Creates a new feed manager
    pub fn new(rss_service: Arc<RssService>, colors: AppColors) -> Self {
        Self {
            visible: true,
            url: String::new(),
            title: String::new(),
            description: String::new(),
            selected_category: None,
            selected_categories: HashSet::new(),
            categories: Vec::new(),
            feeds: Vec::new(),
            colors,
            rss_service,
            url_input: String::new(),
            error_message: None,
            show_preview: false,
            is_fetching: false,
            feed_preview: None,
            is_saving: false,
        }
    }
    
    /// Shows the feed manager dialog
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        if self.visible {
            Window::new("Add Feed")
                .collapsible(false)
                .resizable(true)
                .min_width(400.0)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        // Render URL input section
                        self.render_url_input_section(ui);
                        
                        // Show any error messages
                        self.render_error_message(ui);
                        
                        // Render feed preview and editing section if needed
                        if self.show_preview {
                            self.render_feed_preview(ui);
                        }
                    });
                });
        }

        // Process pending operations
        self.process_fetch_operation()?;
        self.process_save_operation()?;
        
        Ok(())
    }
    
    /// Renders the URL input section
    fn render_url_input_section(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Enter Feed URL").color(self.colors.text));
        ui.horizontal(|ui| {
            ui.add(TextEdit::singleline(&mut self.url_input)
                .hint_text("https://example.com/feed.xml")
                .desired_width(300.0));
            
            // Fetch button to get feed info
            let fetch_clicked = ui.add_enabled(!self.is_fetching, Button::new("Fetch")).clicked();
            if fetch_clicked {
                self.prepare_fetch_operation();
            }
        });
    }
    
    /// Renders error messages if any
    fn render_error_message(&self, ui: &mut Ui) {
        if let Some(error) = &self.error_message {
            ui.label(RichText::new(error).color(self.colors.error));
        }
    }
    
    /// Renders the feed preview section
    fn render_feed_preview(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        ui.label(RichText::new("Feed Information").color(self.colors.text).size(16.0));
        
        // Feed title
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.add(TextEdit::singleline(&mut self.title)
                .desired_width(300.0));
        });
        
        // Category selection
        self.render_category_selection(ui);
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Action buttons
        self.render_action_buttons(ui);
    }
    
    /// Renders the category selection UI
    fn render_category_selection(&mut self, ui: &mut Ui) {
        ui.label("Categories:");
        ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
            for category in &self.categories {
                let mut selected = self.selected_categories.contains(&category.id);
                if ui.checkbox(&mut selected, &category.name).changed() {
                    if selected {
                        self.selected_categories.insert(category.id.clone());
                    } else {
                        self.selected_categories.remove(&category.id);
                    }
                }
            }
        });
    }
    
    /// Renders the action buttons (Save/Cancel)
    fn render_action_buttons(&mut self, ui: &mut Ui) {
        let save_clicked = ui.button("Save").clicked();
        let cancel_clicked = ui.button("Cancel").clicked();
        
        if save_clicked {
            self.prepare_save_operation();
        }
        
        if cancel_clicked {
            self.cancel_operation();
        }
    }
    
    /// Prepares the fetch operation by setting flags
    fn prepare_fetch_operation(&mut self) {
        self.url = self.url_input.clone();
        self.is_fetching = true;
        self.show_preview = false;
        self.feed_preview = None;
        self.error_message = None;
    }
    
    /// Processes the fetch operation if requested
    fn process_fetch_operation(&mut self) -> Result<()> {
        if self.is_fetching && !self.show_preview {
            // Use a runtime to execute the async operation
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(self.fetch_feed_info()) {
                Ok(feed) => {
                    self.handle_fetch_success(feed);
                },
                Err(e) => {
                    self.handle_fetch_error(e);
                }
            }
            self.is_fetching = false;
        }
        Ok(())
    }
    
    /// Handles a successful fetch operation
    fn handle_fetch_success(&mut self, feed: Feed) {
        self.title = feed.title.clone();
        self.feed_preview = Some(feed);
        self.show_preview = true;
        self.error_message = None;
    }
    
    /// Handles a failed fetch operation
    fn handle_fetch_error(&mut self, e: anyhow::Error) {
        error!("Failed to fetch feed: {}", e);
        self.error_message = Some(e.to_string());
    }
    
    /// Prepares the save operation by setting flags
    fn prepare_save_operation(&mut self) {
        self.is_saving = true;
    }
    
    /// Processes the save operation if requested
    fn process_save_operation(&mut self) -> Result<()> {
        if self.is_saving {
            // Use a runtime to execute the async operation
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(self.save_feed()) {
                Ok(_) => {
                    self.handle_save_success();
                },
                Err(e) => {
                    self.handle_save_error(e);
                }
            }
            self.is_saving = false;
        }
        Ok(())
    }
    
    /// Handles a successful save operation
    fn handle_save_success(&mut self) {
        self.reset_form();
    }
    
    /// Handles a failed save operation
    fn handle_save_error(&mut self, e: anyhow::Error) {
        error!("Failed to add feed: {}", e);
        self.error_message = Some(e.to_string());
    }
    
    /// Cancels the current operation
    fn cancel_operation(&mut self) {
        self.reset_form();
    }
    
    /// Resets the form to its initial state
    fn reset_form(&mut self) {
        self.visible = false;
        self.url_input.clear();
        self.error_message = None;
        self.show_preview = false;
        self.selected_categories.clear();
    }
    
    /// Opens the dialog in add mode
    pub fn open_add(&mut self) {
        self.visible = true;
        self.url.clear();
        self.title.clear();
        self.description.clear();
        self.selected_category = None;
        self.selected_categories.clear();
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
        self.selected_categories.clear();
    }
    
    /// Returns whether the dialog is currently open
    pub fn is_open(&self) -> bool {
        self.visible
    }
    
    /// Fetches feed information from the URL
    async fn fetch_feed_info(&mut self) -> Result<Feed> {
        if self.url_input.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        let url = Url::parse(&self.url_input)?;
        // Pass the URL as a string since fetch_feed expects &str
        let feed = self.rss_service.fetch_feed(url.as_str()).await?;
        
        // self.title = feed.title;
        // Feed doesn't have a description field
        self.description = String::new();
        self.url = url.to_string();
        
        Ok(feed)
    }
    
    /// Saves the current feed
    async fn save_feed(&mut self) -> Result<()> {
        if self.url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        let url = Url::parse(&self.url)?;
        
        // For each selected category, create and save a feed
        if self.selected_categories.is_empty() {
            // No categories selected, save with default settings
            let feed = Feed::new(self.title.clone(), url.clone());
            self.rss_service.add_feed(&feed.url.to_string()).await?;
        } else {
            // Process each selected category
            for category_id in &self.selected_categories {
                // Find the category in our list
                if let Some(category) = self.categories.iter().find(|c| &c.id == category_id) {
                    info!("Adding feed '{}' to category '{}'", self.title, category.name);
                    
                    // Create a feed with this category
                    let mut feed = Feed::new(self.title.clone(), url.clone());
                    feed = feed.with_category(category_id.clone());
                    
                    // Add the feed
                    match self.rss_service.add_feed(&feed.url.to_string()).await {
                        Ok(_) => {
                            // If successful, update the feed with the category
                            // This is a workaround since add_feed doesn't directly support categories
                            if let Ok(Some(saved_feed)) = self.rss_service.get_feed_by_url(&feed.url.to_string()).await {
                                let mut updated_feed = saved_feed.clone();
                                updated_feed.category_id = Some(category_id.clone());
                                self.rss_service.update_feed(&updated_feed).await?;
                            }
                        },
                        Err(e) => {
                            error!("Failed to add feed to category {}: {}", category.name, e);
                            return Err(e);
                        }
                    }
                }
            }
        }
        
        // Refresh the feeds list to show the new feeds
        self.refresh().await?;
        
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