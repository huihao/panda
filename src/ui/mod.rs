pub mod components;
pub mod styles;
pub mod theme;
pub mod views;

use anyhow::Result;
use std::sync::Arc;

use crate::base::repository::{ArticleRepository, CategoryRepository, FeedRepository, TagRepository};
use crate::services::rss::RssService;
use crate::services::sync::SyncService;
use crate::services::webview::WebViewService;

pub struct AppContext {
    pub article_repository: Arc<dyn ArticleRepository>,
    pub category_repository: Arc<dyn CategoryRepository>,
    pub feed_repository: Arc<dyn FeedRepository>,
    pub tag_repository: Arc<dyn TagRepository>,
    pub rss_service: Arc<RssService>,
    pub sync_service: Arc<SyncService>,
    pub webview_service: Arc<WebViewService>,
}

impl AppContext {
    pub fn new(
        article_repository: Arc<dyn ArticleRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        feed_repository: Arc<dyn FeedRepository>,
        tag_repository: Arc<dyn TagRepository>,
    ) -> Self {
        let rss_service = Arc::new(RssService::new(
            article_repository.clone(),
            feed_repository.clone(),
            category_repository.clone(),
            tag_repository.clone(),
        ));

        let sync_service = Arc::new(SyncService::new(rss_service.clone()));
        let webview_service = Arc::new(WebViewService::new());

        Self {
            article_repository,
            category_repository,
            feed_repository,
            tag_repository,
            rss_service,
            sync_service,
            webview_service,
        }
    }
}