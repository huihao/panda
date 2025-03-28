use std::sync::Arc;
use egui::{Ui, SidePanel, CentralPanel, TopBottomPanel, Layout, RichText, Menu, ScrollArea};
use anyhow::Result;
use rfd::FileDialog;
use log::{info, error};

use crate::core::{FeedRepository, CategoryRepository, ArticleRepository};
use crate::models::{Article, Feed, Category, Tag};
use crate::services::{RssService, WebViewService, SyncService, OpmlService};
use crate::ui::components::{
    Sidebar, SidebarSelection,
    ArticleList, ArticleSortOrder,
    ArticleViewer,
    FeedManager,
    CategoryManager,
    SettingsDialog,
};
use crate::ui::styles::{AppColors, DEFAULT_PADDING};

/// Main view component that organizes the application layout
pub struct MainView {
    // Repositories
    feed_repository: Arc<dyn FeedRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    article_repository: Arc<dyn ArticleRepository>,
    
    // Services
    rss_service: Arc<RssService>,
    webview_service: Arc<WebViewService>,
    sync_service: Arc<SyncService>,
    opml_service: Arc<OpmlService>,
    
    // UI Components
    sidebar: Sidebar,
    article_list: ArticleList,
    article_viewer: ArticleViewer,
    feed_manager: FeedManager,
    category_manager: CategoryManager,
    settings_dialog: SettingsDialog,
    
    // UI State
    colors: AppColors,
    show_sync_indicator: bool,
    status_message: Option<(String, f32)>, // (message, time_remaining)
}

impl MainView {
    /// Creates a new main view
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        article_repository: Arc<dyn ArticleRepository>,
        rss_service: Arc<RssService>,
        webview_service: Arc<WebViewService>,
        sync_service: Arc<SyncService>,
    ) -> Self {
        Self {
            sidebar: Sidebar::new(feed_repository.clone(), category_repository.clone()),
            article_list: ArticleList::new(article_repository.clone()),
            article_viewer: ArticleViewer::new(article_repository.clone(), webview_service.clone()),
            feed_manager: FeedManager::new(
                feed_repository.clone(),
                category_repository.clone(),
                rss_service.clone(),
            ),
            category_manager: CategoryManager::new(category_repository.clone()),
            settings_dialog: SettingsDialog::new(sync_service.clone()),
            feed_repository,
            category_repository,
            article_repository,
            rss_service,
            webview_service,
            sync_service,
            opml_service: Arc::new(OpmlService::new()),
            colors: AppColors::default(),
            show_sync_indicator: false,
            status_message: None,
        }
    }
    
    /// Renders the main view UI
    pub fn ui(&mut self, ctx: &egui::Context) -> Result<()> {
        self.update_status(ctx.input().predicted_dt);
        
        // Top menu bar
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Feed menu
                Menu::new("Feed", RichText::new("Feed").size(14.0)).show(ui, |ui| {
                    if ui.button("Add Feed...").clicked() {
                        self.feed_manager.open_add();
                    }
                    if ui.button("Import OPML...").clicked() {
                        self.import_opml();
                    }
                    if ui.button("Export OPML...").clicked() {
                        self.export_opml();
                    }
                    ui.separator();
                    if ui.button("Sync All Feeds").clicked() {
                        self.sync_all_feeds();
                    }
                });
                
                // Categories menu
                Menu::new("Categories", RichText::new("Categories").size(14.0)).show(ui, |ui| {
                    if ui.button("Manage Categories...").clicked() {
                        self.category_manager.open();
                    }
                });
                
                // View menu
                Menu::new("View", RichText::new("View").size(14.0)).show(ui, |ui| {
                    ui.menu_button("Sort By", |ui| {
                        if ui.radio_value(
                            &mut self.article_list.sort_order,
                            ArticleSortOrder::NewestFirst,
                            "Newest First"
                        ).clicked() {
                            self.article_list.refresh()?;
                        }
                        if ui.radio_value(
                            &mut self.article_list.sort_order,
                            ArticleSortOrder::OldestFirst,
                            "Oldest First"
                        ).clicked() {
                            self.article_list.refresh()?;
                        }
                    });
                    ui.separator();
                    if ui.button("Settings...").clicked() {
                        self.settings_dialog.open();
                    }
                });
                
                // Show sync indicator if active
                if self.show_sync_indicator {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spinner();
                        ui.label("Syncing feeds...");
                    });
                }
            });
        });
        
        // Status bar
        if let Some((message, _)) = &self.status_message {
            TopBottomPanel::bottom("status_bar")
                .min_height(24.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(message);
                    });
                });
        }
        
        // Left sidebar with feeds and categories
        SidePanel::left("sidebar")
            .resizable(true)
            .min_width(250.0)
            .show(ctx, |ui| {
                if let Some(selection) = self.sidebar.ui(ui)? {
                    self.handle_sidebar_selection(&selection)?;
                }
            });
        
        // Article list panel
        SidePanel::left("article_list")
            .resizable(true)
            .min_width(300.0)
            .show(ctx, |ui| {
                self.render_article_list(ui)?;
            });
        
        // Main content area with article viewer
        CentralPanel::default().show(ctx, |ui| {
            self.article_viewer.ui(ui)?;
        });
        
        // Modal dialogs
        self.feed_manager.show(ui)?;
        self.category_manager.show(ui)?;
        self.settings_dialog.show(ui)?;
        
        Ok(())
    }
    
    /// Renders the article list
    fn render_article_list(&mut self, ui: &mut Ui) -> Result<()> {
        let articles = match self.sidebar.get_selection() {
            Some(SidebarSelection::AllFeeds) => self.article_list.get_all_articles()?,
            Some(SidebarSelection::Favorites) => self.article_list.get_favorite_articles()?,
            Some(SidebarSelection::Feed(feed)) => self.article_list.get_feed_articles(&feed.id)?,
            Some(SidebarSelection::Category(category_id)) => {
                let mut articles = Vec::new();
                if let Ok(feeds) = self.feed_repository.get_feeds_by_category(&category_id) {
                    for feed in feeds {
                        if let Ok(mut feed_articles) = self.article_list.get_feed_articles(&feed.id) {
                            articles.append(&mut feed_articles);
                        }
                    }
                }
                articles
            }
            None => Vec::new(),
        };
        
        if let Some(article_id) = self.article_list.ui(ui, &articles)? {
            self.article_viewer.load_article(&article_id)?;
        }
        
        Ok(())
    }
    
    /// Handles selection changes in the sidebar
    fn handle_sidebar_selection(&mut self, selection: &SidebarSelection) -> Result<()> {
        // Clear current article selection when changing feeds/categories
        self.article_viewer.clear();
        self.article_list.set_selected_article(None);
        
        Ok(())
    }
    
    /// Updates status messages and indicators
    fn update_status(&mut self, dt: f32) {
        // Update status message lifetime
        if let Some((msg, time)) = &mut self.status_message {
            *time -= dt;
            if *time <= 0.0 {
                self.status_message = None;
            }
        }
    }
    
    /// Sets a temporary status message
    fn set_status(&mut self, message: String, duration: f32) {
        self.status_message = Some((message, duration));
    }
    
    /// Imports feeds from an OPML file
    fn import_opml(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("OPML Files", &["opml", "xml"])
            .pick_file()
        {
            match self.opml_service.import(&path) {
                Ok((feeds, categories)) => {
                    // Save categories first
                    for category in categories {
                        if let Err(e) = self.category_repository.save_category(&category) {
                            error!("Failed to save category: {}", e);
                        }
                    }
                    
                    // Then save feeds
                    for feed in feeds {
                        if let Err(e) = self.feed_repository.save_feed(&feed) {
                            error!("Failed to save feed: {}", e);
                        }
                    }
                    
                    self.set_status(format!("Imported {} feeds", feeds.len()), 3.0);
                }
                Err(e) => {
                    error!("Failed to import OPML: {}", e);
                    self.set_status("Failed to import OPML file".to_string(), 3.0);
                }
            }
        }
    }
    
    /// Exports feeds to an OPML file
    fn export_opml(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("OPML Files", &["opml"])
            .save_file()
        {
            match (
                self.feed_repository.get_all_feeds(),
                self.category_repository.get_all_categories(),
            ) {
                (Ok(feeds), Ok(categories)) => {
                    if let Err(e) = self.opml_service.export(&path, &feeds, &categories) {
                        error!("Failed to export OPML: {}", e);
                        self.set_status("Failed to export OPML file".to_string(), 3.0);
                    } else {
                        self.set_status("OPML file exported successfully".to_string(), 3.0);
                    }
                }
                _ => {
                    self.set_status("Failed to export OPML file".to_string(), 3.0);
                }
            }
        }
    }
    
    /// Synchronizes all feeds
    fn sync_all_feeds(&mut self) {
        self.show_sync_indicator = true;
        
        match self.sync_service.sync_all_feeds() {
            Ok(_) => {
                self.set_status("All feeds synchronized".to_string(), 3.0);
            }
            Err(e) => {
                error!("Failed to sync feeds: {}", e);
                self.set_status("Failed to synchronize feeds".to_string(), 3.0);
            }
        }
        
        self.show_sync_indicator = false;
    }
}