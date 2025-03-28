use std::sync::Arc;
use anyhow::Result;
use egui::{Ui, ScrollArea, RichText, Label, TextEdit, Frame};
use log::error;

use crate::core::{FeedRepository, CategoryRepository};
use crate::models::{Feed, Category, CategoryId};
use crate::ui::styles::{AppColors, DEFAULT_PADDING};

/// Sidebar selection state
#[derive(Debug, Clone, PartialEq)]
pub enum SidebarSelection {
    AllFeeds,
    Favorites,
    Feed(Feed),
    Category(CategoryId),
}

/// Sidebar component for feed and category navigation
pub struct Sidebar {
    feed_repository: Arc<dyn FeedRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    colors: AppColors,
    selection: Option<SidebarSelection>,
    feeds: Vec<Feed>,
    categories: Vec<Category>,
    search_query: String,
    expanded_categories: std::collections::HashSet<CategoryId>,
}

impl Sidebar {
    /// Creates a new sidebar
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        category_repository: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            feed_repository,
            category_repository,
            colors: AppColors::default(),
            selection: None,
            feeds: Vec::new(),
            categories: Vec::new(),
            search_query: String::new(),
            expanded_categories: std::collections::HashSet::new(),
        }
    }
    
    /// Renders the sidebar UI
    pub fn ui(&mut self, ui: &mut Ui) -> Result<Option<SidebarSelection>> {
        let mut selected = None;
        
        // Refresh data
        self.feeds = self.feed_repository.get_all_feeds()?;
        self.categories = self.category_repository.get_all_categories()?;
        
        // Search box
        ui.horizontal(|ui| {
            let search_response = TextEdit::singleline(&mut self.search_query)
                .hint_text("Search feeds...")
                .desired_width(ui.available_width())
                .show(ui)
                .response;
            
            if search_response.changed() {
                // Filter will be applied below
            }
        });
        
        ui.add_space(DEFAULT_PADDING);
        
        // Special sections
        if self.selection == Some(SidebarSelection::AllFeeds) {
            ui.horizontal(|ui| {
                ui.add(Label::new(
                    RichText::new("📚 All Articles")
                        .color(self.colors.accent)
                        .size(14.0)
                ));
            });
        } else {
            if ui.add(Label::new(
                RichText::new("📚 All Articles")
                    .color(self.colors.text)
                    .size(14.0)
            )).clicked() {
                self.selection = Some(SidebarSelection::AllFeeds);
                selected = Some(SidebarSelection::AllFeeds);
            }
        }
        
        if self.selection == Some(SidebarSelection::Favorites) {
            ui.horizontal(|ui| {
                ui.add(Label::new(
                    RichText::new("⭐ Favorites")
                        .color(self.colors.accent)
                        .size(14.0)
                ));
            });
        } else {
            if ui.add(Label::new(
                RichText::new("⭐ Favorites")
                    .color(self.colors.text)
                    .size(14.0)
            )).clicked() {
                self.selection = Some(SidebarSelection::Favorites);
                selected = Some(SidebarSelection::Favorites);
            }
        }
        
        ui.add_space(DEFAULT_PADDING);
        
        // Feeds and categories
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                self.render_categories(ui, None, 0, &mut selected)?;
                
                // Show uncategorized feeds
                let uncategorized: Vec<_> = self.feeds.iter()
                    .filter(|f| f.category_id.is_none())
                    .filter(|f| self.matches_search(f))
                    .collect();
                
                if !uncategorized.is_empty() {
                    ui.add_space(DEFAULT_PADDING);
                    ui.add(Label::new(
                        RichText::new("Uncategorized")
                            .color(self.colors.text_dimmed)
                            .size(12.0)
                    ));
                    
                    for feed in uncategorized {
                        self.render_feed(ui, feed, 1, &mut selected);
                    }
                }
            });
        
        Ok(selected)
    }
    
    /// Renders categories recursively
    fn render_categories(
        &mut self,
        ui: &mut Ui,
        parent_id: Option<CategoryId>,
        depth: usize,
        selected: &mut Option<SidebarSelection>,
    ) -> Result<()> {
        let categories: Vec<_> = self.categories.iter()
            .filter(|c| c.parent_id.as_ref() == parent_id.as_ref())
            .collect();
        
        for category in categories {
            let indent = depth * 16.0;
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.add_space(indent);
                
                // Expansion toggle
                let is_expanded = self.expanded_categories.contains(&category.id);
                let has_children = self.categories.iter()
                    .any(|c| c.parent_id.as_ref() == Some(&category.id));
                
                if has_children {
                    let toggle_text = if is_expanded { "▼" } else { "▶" };
                    if ui.add(Label::new(
                        RichText::new(toggle_text)
                            .color(self.colors.text_dimmed)
                            .size(10.0)
                    )).clicked() {
                        if is_expanded {
                            self.expanded_categories.remove(&category.id);
                        } else {
                            self.expanded_categories.insert(category.id.clone());
                        }
                    }
                } else {
                    ui.add_space(14.0); // Width of toggle button
                }
                
                // Category label
                let label_text = format!("📁 {}", category.name);
                if self.selection == Some(SidebarSelection::Category(category.id.clone())) {
                    ui.add(Label::new(
                        RichText::new(label_text)
                            .color(self.colors.accent)
                            .size(14.0)
                    ));
                } else {
                    if ui.add(Label::new(
                        RichText::new(label_text)
                            .color(self.colors.text)
                            .size(14.0)
                    )).clicked() {
                        self.selection = Some(SidebarSelection::Category(category.id.clone()));
                        *selected = Some(SidebarSelection::Category(category.id.clone()));
                    }
                }
            });
            
            // Show child categories and feeds if expanded
            if self.expanded_categories.contains(&category.id) {
                // Render child categories
                self.render_categories(ui, Some(category.id.clone()), depth + 1, selected)?;
                
                // Render feeds in this category
                let category_feeds: Vec<_> = self.feeds.iter()
                    .filter(|f| f.category_id.as_ref() == Some(&category.id))
                    .filter(|f| self.matches_search(f))
                    .collect();
                
                for feed in category_feeds {
                    self.render_feed(ui, feed, depth + 1, selected);
                }
            }
        }
        
        Ok(())
    }
    
    /// Renders a feed item
    fn render_feed(
        &self,
        ui: &mut Ui,
        feed: &Feed,
        depth: usize,
        selected: &mut Option<SidebarSelection>,
    ) {
        let indent = depth * 16.0;
        ui.horizontal(|ui| {
            ui.add_space(indent + 14.0); // Add space for alignment with categories
            
            if let Some(SidebarSelection::Feed(selected_feed)) = &self.selection {
                if selected_feed.id == feed.id {
                    ui.add(Label::new(
                        RichText::new(format!("📰 {}", feed.title))
                            .color(self.colors.accent)
                            .size(14.0)
                    ));
                    return;
                }
            }
            
            if ui.add(Label::new(
                RichText::new(format!("📰 {}", feed.title))
                    .color(self.colors.text)
                    .size(14.0)
            )).clicked() {
                *selected = Some(SidebarSelection::Feed(feed.clone()));
            }
        });
    }
    
    /// Gets the current selection
    pub fn get_selection(&self) -> Option<SidebarSelection> {
        self.selection.clone()
    }
    
    /// Checks if a feed matches the search query
    fn matches_search(&self, feed: &Feed) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        
        let query = self.search_query.to_lowercase();
        feed.title.to_lowercase().contains(&query) ||
            feed.url.to_string().to_lowercase().contains(&query)
    }
}