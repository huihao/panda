use std::sync::Arc;
use anyhow::Result;
use egui::{Ui, Window, TextEdit, ComboBox, Button, RichText};
use url::Url;
use log::{info, error};

use crate::core::{FeedRepository, CategoryRepository};
use crate::models::{Feed, Category, CategoryId};
use crate::services::RssService;
use crate::ui::styles::{AppColors, DEFAULT_PADDING};

/// Feed management dialog component
pub struct FeedManager {
    feed_repository: Arc<dyn FeedRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    rss_service: Arc<RssService>,
    colors: AppColors,
    is_open: bool,
    is_editing: bool,
    url: String,
    title: String,
    selected_category: Option<String>,
    current_feed: Option<Feed>,
    error_message: Option<String>,
    update_interval: u32,
    categories: Vec<Category>,
    is_processing: bool,
}

impl FeedManager {
    /// Creates a new feed manager
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        rss_service: Arc<RssService>,
    ) -> Self {
        Self {
            feed_repository,
            category_repository,
            rss_service,
            colors: AppColors::default(),
            is_open: false,
            is_editing: false,
            url: String::new(),
            title: String::new(),
            selected_category: None,
            current_feed: None,
            error_message: None,
            update_interval: 60,
            categories: Vec::new(),
            is_processing: false,
        }
    }
    
    /// Shows the feed manager dialog
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        if !self.is_open {
            return Ok(());
        }
        
        // Refresh categories list
        self.categories = self.category_repository.get_all_categories()?;
        
        let title = if self.is_editing {
            "Edit Feed"
        } else {
            "Add New Feed"
        };
        
        Window::new(title)
            .collapsible(false)
            .resizable(false)
            .min_width(400.0)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    // Feed URL
                    ui.label("Feed URL:");
                    let url_edit = ui.add(
                        TextEdit::singleline(&mut self.url)
                            .hint_text("Enter RSS/Atom feed URL")
                    );
                    
                    if url_edit.changed() {
                        self.error_message = None;
                        self.title.clear(); // Clear title when URL changes
                    }
                    
                    // Fetch title button
                    if !self.url.is_empty() && !self.is_processing {
                        ui.horizontal(|ui| {
                            if ui.button("📥 Fetch Feed Info").clicked() {
                                self.fetch_feed_info();
                            }
                        });
                    }
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Feed title
                    ui.label("Feed Title (optional):");
                    ui.add(
                        TextEdit::singleline(&mut self.title)
                            .hint_text("Leave empty to use feed's default title")
                    );
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Category selection
                    ui.label("Category:");
                    ComboBox::from_label("")
                        .selected_text(self.selected_category.as_deref().unwrap_or("Uncategorized"))
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(
                                &mut self.selected_category,
                                None,
                                "Uncategorized"
                            ).clicked() {
                                self.selected_category = None;
                            }
                            
                            for category in &self.categories {
                                if ui.selectable_value(
                                    &mut self.selected_category,
                                    Some(category.name.clone()),
                                    &category.name
                                ).clicked() {
                                    self.selected_category = Some(category.name.clone());
                                }
                            }
                        });
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Update interval
                    ui.horizontal(|ui| {
                        ui.label("Update Interval:");
                        ui.add(
                            egui::DragValue::new(&mut self.update_interval)
                                .clamp_range(5..=1440)
                                .speed(5)
                                .suffix(" min")
                        );
                    });
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Error message
                    if let Some(error) = &self.error_message {
                        ui.colored_label(self.colors.error, error);
                        ui.add_space(DEFAULT_PADDING);
                    }
                    
                    // Processing indicator
                    if self.is_processing {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Processing...");
                        });
                    }
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.close();
                        }
                        
                        let button_text = if self.is_editing {
                            "Save Changes"
                        } else {
                            "Add Feed"
                        };
                        
                        if ui.add_enabled(!self.is_processing, Button::new(button_text)).clicked() {
                            match self.save_feed() {
                                Ok(_) => self.close(),
                                Err(e) => self.error_message = Some(e.to_string()),
                            }
                        }
                    });
                });
            });
        
        Ok(())
    }
    
    /// Opens the dialog in add mode
    pub fn open_add(&mut self) {
        self.is_open = true;
        self.is_editing = false;
        self.url.clear();
        self.title.clear();
        self.selected_category = None;
        self.current_feed = None;
        self.error_message = None;
        self.update_interval = 60;
        self.is_processing = false;
    }
    
    /// Opens the dialog in edit mode
    pub fn open_edit(&mut self, feed: Feed) {
        self.is_open = true;
        self.is_editing = true;
        self.url = feed.url.to_string();
        self.title = feed.title.clone();
        self.update_interval = feed.update_interval;
        self.selected_category = feed.category_id.clone().map(|_| feed.category_name.unwrap_or_default());
        self.current_feed = Some(feed);
        self.error_message = None;
        self.is_processing = false;
    }
    
    /// Closes the dialog
    pub fn close(&mut self) {
        self.is_open = false;
        self.is_editing = false;
        self.url.clear();
        self.title.clear();
        self.selected_category = None;
        self.current_feed = None;
        self.error_message = None;
        self.is_processing = false;
    }
    
    /// Returns whether the dialog is currently open
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    
    /// Fetches feed information from the URL
    fn fetch_feed_info(&mut self) {
        self.is_processing = true;
        self.error_message = None;
        
        match Url::parse(&self.url) {
            Ok(url) => {
                match self.rss_service.fetch_feed_info(&url.to_string()) {
                    Ok(Some(feed_info)) => {
                        if self.title.is_empty() {
                            self.title = feed_info.title;
                        }
                        self.error_message = None;
                    }
                    Ok(None) => {
                        self.error_message = Some("Could not find feed information".to_string());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to fetch feed: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Invalid URL: {}", e));
            }
        }
        
        self.is_processing = false;
    }
    
    /// Saves the current feed
    fn save_feed(&mut self) -> Result<()> {
        // Validate URL
        if self.url.is_empty() {
            return Err(anyhow::anyhow!("Feed URL is required"));
        }
        
        let url = Url::parse(&self.url)
            .map_err(|e| anyhow::anyhow!("Invalid URL: {}", e))?;
        
        // Get feed category ID if selected
        let category_id = if let Some(category_name) = &self.selected_category {
            self.categories.iter()
                .find(|c| &c.name == category_name)
                .map(|c| c.id.clone())
        } else {
            None
        };
        
        self.is_processing = true;
        
        // Create or update feed
        let feed = if let Some(mut existing_feed) = self.current_feed.clone() {
            // Update existing feed
            existing_feed.url = url;
            if !self.title.is_empty() {
                existing_feed.title = self.title.clone();
            }
            existing_feed.category_id = category_id.clone();
            existing_feed.category_name = self.selected_category.clone();
            existing_feed.update_interval = self.update_interval;
            
            self.feed_repository.update_feed(&existing_feed)?;
            existing_feed
        } else {
            // Fetch feed info if title not provided
            let feed_info = if self.title.is_empty() {
                self.rss_service.fetch_feed_info(&url.to_string())?
            } else {
                None
            };
            
            // Create new feed
            let feed = Feed::new(
                if !self.title.is_empty() {
                    self.title.clone()
                } else if let Some(info) = feed_info {
                    info.title
                } else {
                    url.to_string()
                },
                url,
            );
            
            let mut feed = feed;
            feed.category_id = category_id;
            feed.category_name = self.selected_category.clone();
            feed.update_interval = self.update_interval;
            
            self.feed_repository.save_feed(&feed)?;
            feed
        };
        
        // Initial fetch
        if let Err(e) = self.rss_service.sync_feed(&feed) {
            error!("Failed to fetch initial feed content: {}", e);
        }
        
        self.is_processing = false;
        Ok(())
    }
}