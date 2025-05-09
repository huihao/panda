use std::sync::Arc;
use egui::{Button, Context, TopBottomPanel, RichText, Color32, CentralPanel, SidePanel, Window};
use crate::ui::AppContext;
use crate::models::category::CategoryId;
use crate::models::article::ArticleId;
use crate::ui::components::*;
use crate::ui::styles::AppColors;
use std::time::{Duration, Instant};
use anyhow::Result;
use eframe::App;
use log::{info, warn, error};

pub struct MainView {
    app_context: AppContext,
    
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
    status_message: Option<(String, Instant)>,
    selected_article: Option<ArticleId>,
    show_categories: bool,
    show_settings: bool,
    show_feed_manager: bool,
}

impl MainView {
    pub fn new(app_context: AppContext) -> Self {
        let colors = AppColors::default();
        
        // Initialize components using AppContext, which now manages async operations safely
        let sidebar = app_context.init_sidebar();
        
        // Initialize other components (these don't have the same Tokio runtime issue)
        let article_list = ArticleList::new(
            app_context.article_repository.clone(),
            app_context.rss_service.clone(), // Properly pass the RssService as required by ArticleList
            colors.clone(),
        );
        
        let article_viewer = ArticleViewer::new(
            app_context.article_repository.clone(),
            app_context.webview_service.clone(),
            app_context.rss_service.clone(),
            colors.clone(),
        );
        
        let feed_manager = FeedManager::new(
            app_context.rss_service.clone(),
            colors.clone(),
        );
        
        let category_manager = CategoryManager::new(
            app_context.category_repository.clone(),
            colors.clone(),
        );
        
        let settings_dialog = SettingsDialog::new(
            app_context.sync_service.clone(),
            colors.clone(),
        );
        
        Self {
            sidebar,
            article_list,
            article_viewer,
            feed_manager,
            category_manager,
            settings_dialog,
            app_context,
            colors,
            show_sync_indicator: false,
            status_message: None,
            selected_article: None,
            show_categories: false,
            show_settings: false,
            show_feed_manager: false
        }
    }

    pub fn update(&mut self, ctx: &Context) -> Result<()> {
        if let Some(article_id) = self.selected_article.clone() {
            // TODO: This needs to be handled through the background thread mechanism
            // For now, we'll just display any cached article data
            self.set_status_message("Loading article...".to_string());
        }

        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(Button::new("FeedManage")).clicked() {
                    self.show_feed_manager = !self.show_feed_manager;
                }
                if ui.add(Button::new("Categories")).clicked() {
                    self.show_categories = !self.show_categories;
                }

                if ui.add(Button::new("Settings")).clicked() {
                    self.show_settings = !self.show_settings;
                }

                if ui.add(Button::new("Sync All")).clicked() {
                    self.sync_all();
                }
            });
        });

        // Display the feed manager window when show_feed_manager is true
        if self.show_feed_manager {
            Window::new("Feed Manager")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    // Add a close button at the top
                    if ui.button("Close").clicked() {
                        self.show_feed_manager = false;
                    }
                    
                    // Show feed manager UI
                    if let Err(e) = self.feed_manager.show(ui) {
                        error!("Error rendering feed manager: {}", e);
                        self.set_status_message(format!("Error displaying feed manager: {}", e));
                    }
                });
        }

        if let Some((msg, time)) = &self.status_message {
            if time.elapsed() > Duration::from_secs(5) {
                self.status_message = None;
            } else {
                TopBottomPanel::bottom("status").show(ctx, |ui| {
                    ui.label(RichText::new(msg).color(Color32::WHITE));
                });
            }
        }

        Ok(())
    }

    fn sync_all(&mut self) {
        self.show_sync_indicator = true;
        self.set_status_message("Starting sync...".to_string());
        
        // The actual sync should now be handled through a background thread or queue
        // For now, just indicate success as we've moved away from direct runtime usage
        // TODO: Implement a proper background sync mechanism
        self.set_status_message("Sync request submitted".to_string());
        self.show_sync_indicator = false;
    }

    fn set_status_message(&mut self, message: String) {
        info!("Status: {}", message);
        self.status_message = Some((message, Instant::now()));
    }

    pub fn select_article(&mut self, article_id: ArticleId) {
        self.selected_article = Some(article_id);
    }

    pub fn select_category(&mut self, category_id: CategoryId) {
        self.sidebar.select_category(category_id);
    }
}

impl App for MainView {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Use the existing update method and handle any errors
        if let Err(e) = self.update(ctx) {
            // Log error or show in UI
            error!("Error in update: {}", e);
            self.set_status_message(format!("Error: {}", e));
        }
        
        // Set up main UI layout
        SidePanel::left("sidebar_panel").show(ctx, |ui| {
            if let Err(e) = self.sidebar.ui(ui) {
                error!("Error rendering sidebar: {}", e);
            }
        });

        // Main content area - only one CentralPanel should exist
        CentralPanel::default().show(ctx, |ui| {
            // Depending on what's selected in the sidebar, show either article list or article
            if self.selected_article.is_some() {
                if let Err(e) = self.article_viewer.ui(ui) {
                    error!("Error rendering article viewer: {}", e);
                }
            } else {
                if let Err(e) = self.article_list.ui(ui) {
                    error!("Error rendering article list: {}", e);
                }
            }
        });
        
        // Render settings dialog if visible
        if self.show_settings {
            if let Err(e) = self.settings_dialog.show(ctx) {
                error!("Error rendering settings dialog: {}", e);
            }
        }

        // Wrap the category_manager.show call in a Window to provide a UI context
        if self.show_categories {
            // Create a temporary window to provide a UI context for the category manager
            Window::new("Category Manager")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Err(e) = self.category_manager.show(ui) {
                        error!("Error rendering category manager: {}", e);
                    }
                });
        }
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Clean shutdown of the application
        info!("Application shutting down");
        self.app_context.shutdown();
    }
}