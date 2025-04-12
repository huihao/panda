use egui::{Button, Context as EguiContext, Label, RichText, ScrollArea, TextEdit, Ui};
use std::sync::Arc;
use std::collections::HashMap;
use log::{error, warn};
use anyhow::Result;

use crate::base::repository::{FeedRepository, CategoryRepository};
use crate::models::category::{Category, CategoryId};
use crate::models::feed::Feed;
use crate::ui::styles::{AppColors, DEFAULT_PADDING};

/// Data model specifically for the Sidebar component
/// This implements SRP by separating the UI state from the data model
#[derive(Debug, Clone, Default)]
pub struct SidebarData {
    pub categories_by_parent: HashMap<Option<CategoryId>, Vec<Category>>,
    pub feeds_by_category: HashMap<Option<CategoryId>, Vec<Feed>>,
    pub is_loading: bool,
    pub last_error: Option<String>,
}

impl SidebarData {
    pub fn new() -> Self {
        Self {
            categories_by_parent: HashMap::new(),
            feeds_by_category: HashMap::new(),
            is_loading: false,
            last_error: None,
        }
    }

    /// Determine if data is available or needs to be loaded
    pub fn has_data_for(&self, parent_id: &Option<CategoryId>) -> bool {
        self.categories_by_parent.contains_key(parent_id)
    }
    
    /// Get categories for a parent ID from the cached data
    pub fn get_categories(&self, parent_id: &Option<CategoryId>) -> Vec<Category> {
        self.categories_by_parent.get(parent_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get feeds for a category ID from the cached data
    pub fn get_feeds(&self, category_id: &Option<CategoryId>) -> Vec<Feed> {
        self.feeds_by_category.get(category_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Method to update the data model with new categories
    pub fn update_categories(&mut self, parent_id: Option<CategoryId>, categories: Vec<Category>) {
        self.categories_by_parent.insert(parent_id, categories);
    }
    
    /// Method to update the data model with new feeds
    pub fn update_feeds(&mut self, category_id: Option<CategoryId>, feeds: Vec<Feed>) {
        self.feeds_by_category.insert(category_id, feeds);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SidebarState {
    pub selection: Option<SidebarSelection>,
    pub search_query: String,
    pub expanded_categories: Vec<CategoryId>,
    pub data_load_requested: Vec<Option<CategoryId>>, // Track IDs that need data loading
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            selection: None,
            search_query: String::new(),
            expanded_categories: Vec::new(),
            data_load_requested: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarSelection {
    AllFeeds,
    Favorites,
    Feed(Feed),
    Category(Category),
}

pub struct Sidebar {
    feed_repository: Arc<dyn FeedRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    state: SidebarState,
    data: SidebarData,
    colors: AppColors,
}

impl Sidebar {
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        category_repository: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            feed_repository,
            category_repository,
            state: SidebarState::default(),
            data: SidebarData::new(),
            colors: AppColors::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Result<Option<SidebarSelection>> {
        let mut new_selection = None;

        // Search box
        ui.horizontal(|ui| {
            ui.label("Search");
            if ui.text_edit_singleline(&mut self.state.search_query).changed() {
                // Search logic will be handled by the filtering below
            }
        });

        ui.add_space(DEFAULT_PADDING);

        // Special sections
        if ui.add(Button::new(RichText::new("📚 All Articles").color(self.colors.text))).clicked() {
            new_selection = Some(SidebarSelection::AllFeeds);
        }

        if ui.add(Button::new(RichText::new("⭐ Favorites").color(self.colors.text))).clicked() {
            new_selection = Some(SidebarSelection::Favorites);
        }

        ui.add_space(DEFAULT_PADDING);

        // Display loading message if data is being loaded
        if self.data.is_loading {
            ui.label(RichText::new("Loading...").color(self.colors.text));
        }

        // Display any error message
        if let Some(error) = &self.data.last_error {
            ui.label(RichText::new(error).color(egui::Color32::from_rgb(255, 80, 80)));
        }

        // Request data loading for root level (None parent) if needed
        let root_parent_id = None;
        if !self.data.has_data_for(&root_parent_id) && 
           !self.state.data_load_requested.contains(&root_parent_id) {
            self.state.data_load_requested.push(root_parent_id.clone());
        }

        // Categories and feeds
        ScrollArea::vertical().show(ui, |ui| {
            self.render_categories(ui, root_parent_id, 0, &mut new_selection);
        });

        if let Some(selection) = new_selection.clone() {
            self.state.selection = Some(selection);
        }

        Ok(new_selection)
    }

    /// Synchronous method to render categories from the cached data
    fn render_categories(
        &mut self,
        ui: &mut Ui,
        parent_id: Option<CategoryId>,
        depth: i32,
        selection: &mut Option<SidebarSelection>
    ) {
        let categories = self.data.get_categories(&parent_id);
        
        for category in categories {
            let indent = "  ".repeat(depth as usize);
            let text = format!("{}{} {}", indent, "📁", category.name);
            
            if ui.add(Button::new(RichText::new(&text).color(self.colors.text))).clicked() {
                *selection = Some(SidebarSelection::Category(category.clone()));
            }

            // Only show feeds for expanded categories
            if self.state.expanded_categories.contains(&category.id) {
                // Check if we need to request data for this category
                if !self.data.has_data_for(&Some(category.id.clone())) && 
                   !self.state.data_load_requested.contains(&Some(category.id.clone())) {
                    self.state.data_load_requested.push(Some(category.id.clone()));
                }
                
                let feeds = self.data.get_feeds(&Some(category.id.clone()));
                for feed in feeds {
                    self.render_feed(ui, &feed, depth + 1, selection);
                }
            }
        }

        // Show uncategorized feeds at root level
        if parent_id.is_none() {
            let feeds = self.data.get_feeds(&None);
            for feed in feeds {
                self.render_feed(ui, &feed, depth, selection);
            }
        }
    }

    fn render_feed(
        &self,
        ui: &mut Ui,
        feed: &Feed,
        depth: i32,
        selection: &mut Option<SidebarSelection>
    ) {
        let indent = "  ".repeat(depth as usize);
        let text = format!("{}{} {}", indent, "📰", feed.title);
        
        if ui.add(Button::new(RichText::new(&text).color(self.colors.text))).clicked() {
            *selection = Some(SidebarSelection::Feed(feed.clone()));
        }
    }
    
    /// Method to update the cached data with new information from the async thread
    /// This is called from the async thread, not the UI thread
    pub async fn update_data_async(&mut self) -> Result<bool> {
        if self.state.data_load_requested.is_empty() {
            return Ok(false); // No data needs loading
        }
        
        self.data.is_loading = true;
        
        // Take the IDs that need loading
        let ids_to_load = std::mem::take(&mut self.state.data_load_requested);
        let mut updated = false;
        
        for parent_id in ids_to_load {
            // Load categories
            match self.category_repository.get_categories_by_parent(&parent_id).await {
                Ok(categories) => {
                    self.data.update_categories(parent_id.clone(), categories);
                    updated = true;
                },
                Err(e) => {
                    self.data.last_error = Some(format!("Failed to load categories: {}", e));
                    return Err(e);
                }
            }
            
            // Load feeds for this category
            match self.get_feeds_for_category(&parent_id).await {
                Ok(feeds) => {
                    self.data.update_feeds(parent_id, feeds);
                    updated = true;
                },
                Err(e) => {
                    self.data.last_error = Some(format!("Failed to load feeds: {}", e));
                    return Err(e);
                }
            }
        }
        
        self.data.is_loading = false;
        Ok(updated)
    }

    /// Helper method to get feeds by category ID that properly handles the None case
    /// This follows the Adapter pattern to convert between incompatible interfaces
    async fn get_feeds_for_category(&self, category_id_opt: &Option<CategoryId>) -> Result<Vec<Feed>> {
        match category_id_opt {
            Some(category_id) => {
                // When we have a category ID, use it directly
                self.feed_repository.get_feeds_by_category(category_id).await
            },
            None => {
                // For uncategorized feeds, we need to fetch all feeds first and filter
                let all_feeds = self.feed_repository.get_all_feeds().await?;
                
                // Filter to only include feeds without a category
                let uncategorized_feeds = all_feeds.into_iter()
                    .filter(|feed| feed.category_id.is_none())
                    .collect();
                
                Ok(uncategorized_feeds)
            }
        }
    }

    pub fn get_selection(&self) -> Option<SidebarSelection> {
        self.state.selection.clone()
    }

    pub fn clear_selection(&mut self) {
        self.state.selection = None;
    }

    pub fn toggle_category(&mut self, category_id: CategoryId) {
        if let Some(pos) = self.state.expanded_categories.iter().position(|id| *id == category_id) {
            self.state.expanded_categories.remove(pos);
        } else {
            self.state.expanded_categories.push(category_id);
        }
    }

    pub fn select_category(&mut self, category_id: CategoryId) {
        // Check if we already have this category in our data
        for categories in self.data.categories_by_parent.values() {
            for category in categories {
                if category.id == category_id {
                    self.state.selection = Some(SidebarSelection::Category(category.clone()));
                    return;
                }
            }
        }
        
        // If we don't have the category yet, request it to be loaded
        self.state.data_load_requested.push(Some(category_id));
    }
    
    /// Check if there's data that needs to be loaded asynchronously
    pub fn needs_data_loading(&self) -> bool {
        !self.state.data_load_requested.is_empty()
    }
}