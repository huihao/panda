use std::sync::Arc;
use crate::services::rss::RssService;
use crate::services::sync::SyncService;
use crate::services::webview::WebViewService;
use crate::base::repository::{FeedRepository, CategoryRepository, ArticleRepository};
use crate::ui::styles::AppColors;

pub struct Context {
    pub rss_service: Arc<RssService>,
    pub colors: AppColors,
}

impl Context {
    pub fn new(rss_service: Arc<RssService>) -> Self {
        Self {
            rss_service,
            colors: AppColors::default(),
        }
    }
}

// Create a more comprehensive AppContext struct that matches what MainView expects
pub struct AppContext {
    pub feed_repository: Arc<dyn FeedRepository>,
    pub category_repository: Arc<dyn CategoryRepository>,
    pub article_repository: Arc<dyn ArticleRepository>,
    pub rss_service: Arc<RssService>,
    pub webview_service: Arc<WebViewService>,
    pub sync_service: Arc<SyncService>,
}

impl AppContext {
    // Constructor for creating an AppContext from individual components
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        article_repository: Arc<dyn ArticleRepository>,
        rss_service: Arc<RssService>,
        webview_service: Arc<WebViewService>,
        sync_service: Arc<SyncService>,
    ) -> Self {
        Self {
            feed_repository,
            category_repository,
            article_repository,
            rss_service,
            webview_service,
            sync_service,
        }
    }
}