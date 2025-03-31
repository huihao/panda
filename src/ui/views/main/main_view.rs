use std::sync::Arc;
use egui::{Context, RichText, Color32, TopBottomPanel, Button};
use anyhow::Result;
use rfd::FileDialog;
use log::{info, error};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::base::repository::{FeedRepository, CategoryRepository, ArticleRepository};
use crate::models::{Feed, Category, Tag};
use crate::models::article::{Article, ArticleId};
use crate::models::category::CategoryId;
use crate::models::feed::FeedId;
use crate::services::{RssService, WebViewService, SyncService, OpmlService};
use crate::ui::components::{
    Sidebar, SidebarSelection,
    ArticleList, ArticleSortOrder,
    ArticleViewer,
    FeedManager,
    CategoryManager,
    SettingsDialog,
};
use crate::ui::styles::AppColors;

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
    show_categories: bool,
    show_settings: bool,
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
            show_categories: false,
            show_settings: false,
        }
    }
    
    /// Shows the main view
    pub async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        // Render sidebar and central panel
        self.sidebar.show(ctx).await?;
        
        // Show selected article if any
        if let Some(article_id) = self.selected_article {
            if let Ok(Some(article)) = ctx.rss_service.get_article(&article_id).await {
                self.article_viewer.show(&article)?;
            }
        }

        // Show buttons
        if Button::new("Categories").clicked() {
            self.show_categories = true;
        }
        if Button::new("Settings").clicked() {
            self.show_settings = true;
        }
        if Button::new("Sync All").clicked() {
            if let Err(e) = self.sync_all().await {
                log::error!("Failed to sync all feeds: {}", e);
            }
        }

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
                let articles = self.rss_service.get_favorite_articles().await?;
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
    async fn handle_article_selection(&mut self, article_id: ArticleId) -> Result<()> {
        if let Ok(Some(article)) = self.rss_service.get_article(&article_id).await {
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
    
    /// Syncs all feeds
    async fn sync_all(&mut self) -> Result<()> {
        self.rss_service.sync_all_feeds().await?;
        self.sidebar.refresh().await?;
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