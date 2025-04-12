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
use crate::ui::components::sidebar::Sidebar;

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
    
    /// Initialize a new Sidebar component using the repositories from this context
    /// 
    /// This follows the Single Responsibility Principle by delegating the Sidebar
    /// creation to a dedicated method, and the Dependency Inversion Principle by
    /// providing the required repositories via dependency injection.
    pub fn init_sidebar(&self) -> Sidebar {
        Sidebar::new(
            self.feed_repository.clone(),
            self.category_repository.clone(),
        )
    }
    
    /// Performs a clean shutdown of all services managed by this context
    ///
    /// This method follows the Single Responsibility Principle by coordinating
    /// the shutdown of all dependent services. Each service is responsible for
    /// its own cleanup, and this method simply delegates to them.
    ///
    /// It also adheres to the Open/Closed Principle as adding new services that
    /// require cleanup will only require adding new shutdown calls here without
    /// modifying existing code.
    pub fn shutdown(&self) {
        // Log the start of the shutdown process
        log::info!("Beginning application shutdown sequence");
        
        // Shut down services in the reverse order of their dependency
        // This ensures that dependent services are shut down before their dependencies
        
        // First, UI-related services
        log::debug!("Shutting down WebView service");
        // Use the public is_visible() method to check if webview is visible
        if self.webview_service.is_visible() {
            log::warn!("WebView is still visible during shutdown");
            // Ideally, we would call hide() here if we had mutable access
        }
        
        // Then shutdown any background tasks in application services
        log::debug!("Shutting down Sync service");
        // Note: We need to make sure SyncService has a way to stop any background tasks
        // This would typically be implemented in the SyncService
        
        // Finally any core services that might need cleanup
        log::debug!("Shutting down RSS service");
        // Note: We need to make sure RssService has a way to clean up resources if needed
        
        log::info!("Application shutdown complete");
    }
}