use std::sync::Arc;
use egui::{Ui, SidePanel, CentralPanel, TopBottomPanel, Layout, RichText, menu, ScrollArea, Color32, Context};
use anyhow::Result;
use rfd::FileDialog;
use log::{info, error};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::base::repository::{FeedRepository, CategoryRepository, ArticleRepository};
use crate::models::{Article, Feed, Category, Tag};
use crate::models::article::ArticleId;
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
use crate::ui::{
    components::{sidebar::Sidebar, article_viewer::ArticleViewer},
    context::Context,
    theme::AppColors,
};
use crate::models::{article::Article, article::ArticleId, category::CategoryId};
use egui::{Context as EguiContext, Ui};

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
    status_message: Option<(String, Instant)>,
    selected_article: Option<ArticleId>,
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
        let colors = AppColors::default();
        Self {
            sidebar: Sidebar::new(feed_repository.clone(), category_repository.clone()),
            article_list: ArticleList::new(article_repository.clone(), rss_service.clone(), colors.clone()),
            article_viewer: ArticleViewer::new(
                article_repository.clone(),
                webview_service.clone(),
                rss_service.clone(),
                colors.clone(),
            ),
            feed_manager: FeedManager::new(rss_service.clone(), colors.clone()),
            category_manager: CategoryManager::new(category_repository.clone(), colors.clone()),
            settings_dialog: SettingsDialog::new(sync_service.clone(), colors.clone()),
            feed_repository,
            category_repository,
            article_repository,
            rss_service,
            webview_service,
            sync_service,
            opml_service: Arc::new(OpmlService::new(rss_service.clone())),
            colors,
            show_sync_indicator: false,
            status_message: None,
            selected_article: None,
        }
    }
    
    /// Renders the main view UI
    pub async fn show(&mut self, ctx: &Context) -> Result<()> {
        egui::SidePanel::left("sidebar")
            .show(ctx, |ui| {
                self.sidebar.show(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(article_id) = self.selected_article {
                if let Ok(Some(article)) = ctx.rss_service.get_article(&article_id).await {
                    self.article_viewer.show(ui);
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select an article to view");
                });
            }
        });

        egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Categories").clicked() {
                        self.category_manager.show(ui);
                    }
                    if ui.button("Settings").clicked() {
                        self.settings_dialog.show(ui);
                    }
                    if ui.button("Sync All").clicked() {
                        if let Err(e) = self.sync_all().await {
                            log::error!("Error syncing feeds: {}", e);
                        }
                    }
                });
            });

        Ok(())
    }
    
    /// Handles selection changes in the sidebar
    async fn handle_sidebar_selection(&mut self, selection: String) -> Result<()> {
        match selection.as_str() {
            "all" => {
                let articles = self.rss_service.get_all_articles().await?;
                self.article_list.set_articles(articles);
            }
            "favorites" => {
                let articles = self.rss_service.get_favorited_articles().await?;
                self.article_list.set_articles(articles);
            }
            "unread" => {
                let articles = self.rss_service.get_unread_articles().await?;
                self.article_list.set_articles(articles);
            }
            feed_id if feed_id.starts_with("feed:") => {
                let feed_id = FeedId(feed_id[5..].to_string());
                let articles = self.rss_service.get_articles_by_feed(&feed_id).await?;
                self.article_list.set_articles(articles);
            }
            category_id if category_id.starts_with("category:") => {
                let category_id = CategoryId(category_id[9..].to_string());
                let feeds = self.rss_service.get_feeds_by_category(&category_id).await?;
                let mut articles = Vec::new();
                for feed in feeds {
                    if let Ok(feed_articles) = self.rss_service.get_articles_by_feed(&feed.id).await {
                        articles.extend(feed_articles);
                    }
                }
                self.article_list.set_articles(articles);
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handles article selection
    fn handle_article_selection(&mut self, article_id: ArticleId) -> Result<()> {
        if let Some(article) = self.rss_service.get_article(&article_id).await? {
            self.article_viewer.set_article(article);
        }
        Ok(())
    }
    
    /// Updates status messages and indicators
    fn update_status(&mut self, ctx: &Context) {
        if let Some((msg, time)) = &mut self.status_message {
            if time.elapsed() > Duration::from_secs(5) {
                self.status_message = None;
            } else {
                TopBottomPanel::bottom("status").show(ctx, |ui| {
                    ui.label(RichText::new(msg.to_string()).color(Color32::WHITE));
                });
            }
        }
    }
    
    /// Sets a temporary status message
    fn set_status_message(&mut self, message: String) {
        self.status_message = Some((message, Instant::now()));
    }
    
    /// Synchronizes all feeds
    pub async fn sync_all(&mut self) -> Result<()> {
        self.show_sync_indicator = true;
        self.rss_service.fetch_all_feeds().await?;
        self.sidebar.refresh().await?;
        self.set_status_message("Sync completed successfully".to_string());
        self.show_sync_indicator = false;
        Ok(())
    }
    
    async fn refresh(&mut self) -> Result<()> {
        self.sidebar.refresh().await?;
        Ok(())
    }

    pub fn select_article(&mut self, article_id: ArticleId) {
        self.selected_article = Some(article_id);
    }

    pub fn select_category(&mut self, category_id: CategoryId) {
        self.sidebar.select_category(category_id);
    }
}