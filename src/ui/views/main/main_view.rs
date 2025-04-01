use std::sync::Arc;
use egui::{Button, Context, TopBottomPanel, RichText, Color32, CentralPanel, SidePanel};
use tokio::runtime::Runtime;
use crate::ui::AppContext;
use crate::models::category::CategoryId;
use crate::models::article::ArticleId;
use crate::ui::components::*;
use crate::ui::styles::AppColors;
use std::time::{Duration, Instant};
use anyhow::Result;
use eframe::App;

pub struct MainView {
    app_context: AppContext,
    runtime: Runtime,
    
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
}

impl MainView {
    pub fn new(app_context: AppContext) -> Self {
        let colors = AppColors::default();
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        
        Self {
            sidebar: Sidebar::new(
                app_context.feed_repository.clone(),
                app_context.category_repository.clone(),
            ),
            article_list: ArticleList::new(
                app_context.article_repository.clone(),
                app_context.rss_service.clone(),
                colors.clone(),
            ),
            article_viewer: ArticleViewer::new(
                app_context.article_repository.clone(),
                app_context.webview_service.clone(),
                app_context.rss_service.clone(),
                colors.clone(),
            ),
            feed_manager: FeedManager::new(
                app_context.rss_service.clone(),
                colors.clone(),
            ),
            category_manager: CategoryManager::new(
                app_context.category_repository.clone(),
                colors.clone(),
            ),
            settings_dialog: SettingsDialog::new(
                app_context.sync_service.clone(),
                colors.clone(),
            ),
            runtime,
            app_context,
            colors,
            show_sync_indicator: false,
            status_message: None,
            selected_article: None,
            show_categories: false,
            show_settings: false,
        }
    }

    pub fn update(&mut self, ctx: &Context) -> Result<()> {
        if let Some(article_id) = self.selected_article.clone() {
            if let Ok(Some(article)) = self.runtime.block_on(self.app_context.rss_service.get_article(&article_id)) {
                self.article_viewer.set_article(article);
            }
        }

        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
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
        let sync_service = self.app_context.sync_service.clone();
        
        if let Err(e) = self.runtime.block_on(async move {
            sync_service.sync_all().await
        }) {
            self.set_status_message(format!("Sync failed: {}", e));
        } else {
            self.set_status_message("Sync completed successfully".to_string());
        }
        self.show_sync_indicator = false;
    }

    fn set_status_message(&mut self, message: String) {
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
            eprintln!("Error in update: {}", e);
            self.set_status_message(format!("Error: {}", e));
        }
        
        // Set up main UI layout
        SidePanel::left("sidebar_panel").show(ctx, |ui| {
            if let Err(e) = self.sidebar.ui(ui) {
                eprintln!("Error rendering sidebar: {}", e);
            }
        });

        // Main content area - only one CentralPanel should exist
        CentralPanel::default().show(ctx, |ui| {
            // Depending on what's selected in the sidebar, show either article list or article
            if self.selected_article.is_some() {
                if let Err(e) = self.article_viewer.ui(ui) {
                    eprintln!("Error rendering article viewer: {}", e);
                }
            } else {
                if let Err(e) = self.article_list.ui(ui) {
                    eprintln!("Error rendering article list: {}", e);
                }
            }
            
            // Also allow the feed manager to show itself if needed
            // It manages its own Window internally so we just pass the UI context
            if let Err(e) = self.feed_manager.show(ui) {
                eprintln!("Error rendering feed manager: {}", e);
            }
        });
        
        // Render settings dialog if visible
        if self.show_settings {
            if let Err(e) = self.settings_dialog.show(ctx) {
                eprintln!("Error rendering settings dialog: {}", e);
            }
        }
        
        // Render category manager dialog if visible
        // Fix: Wrap the category_manager.show call in a Window to provide a UI context
        if self.show_categories {
            // Create a temporary window to provide a UI context for the category manager
            egui::Window::new("Category Manager")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Err(e) = self.category_manager.show(ui) {
                        eprintln!("Error rendering category manager: {}", e);
                    }
                });
        }
    }
}