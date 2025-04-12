use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use log::{info, warn, error};
use anyhow::Result;

use crate::services::rss::RssService;
use crate::services::sync::SyncService;
use crate::services::webview::WebViewService;
use crate::base::repository::{FeedRepository, CategoryRepository, ArticleRepository, TagRepository};
use crate::ui::styles::AppColors;
use crate::ui::components::sidebar::Sidebar;

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

// A data consumer interested in background updates
pub trait DataConsumer: Send + Sync {
    fn needs_update(&self) -> bool;
    fn update(&mut self) -> Result<()>;
}

// Create a more comprehensive AppContext struct that matches what MainView expects
pub struct AppContext {
    // Repositories
    pub feed_repository: Arc<dyn FeedRepository>,
    pub category_repository: Arc<dyn CategoryRepository>,
    pub article_repository: Arc<dyn ArticleRepository>,
    pub tag_repository: Arc<dyn TagRepository>,
    
    // Services
    pub rss_service: Option<Arc<RssService>>,
    pub webview_service: Option<Arc<WebViewService>>,
    pub sync_service: Option<Arc<SyncService>>,
    
    // Background runtime
    background_runtime: Option<Runtime>,
    
    // UI components that need data
    sidebar: Arc<Mutex<Option<Sidebar>>>,
    
    // Status
    is_loading: Arc<Mutex<bool>>,
    is_shutting_down: Arc<Mutex<bool>>,
}

impl AppContext {
    // Constructor for creating an AppContext with just repositories
    pub fn new(
        article_repository: Arc<dyn ArticleRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        feed_repository: Arc<dyn FeedRepository>,
        tag_repository: Arc<dyn TagRepository>,
    ) -> Self {
        // Create a tokio runtime to handle background data loading
        let background_runtime = Runtime::new().ok();
        
        Self {
            feed_repository,
            category_repository,
            article_repository,
            tag_repository,
            rss_service: None,
            webview_service: None,
            sync_service: None,
            background_runtime,
            sidebar: Arc::new(Mutex::new(None)),
            is_loading: Arc::new(Mutex::new(false)),
            is_shutting_down: Arc::new(Mutex::new(false)),
        }
    }
    
    // Initialize the sidebar component
    pub fn init_sidebar(&self) -> Sidebar {
        let sidebar = Sidebar::new(
            self.feed_repository.clone(),
            self.category_repository.clone(),
        );
        
        // Store the sidebar for background updates
        if let Ok(mut sidebar_guard) = self.sidebar.lock() {
            *sidebar_guard = Some(sidebar.clone());
        }
        
        // Start the background data loading thread
        self.start_background_updates();
        
        sidebar
    }
    
    // Start background updates to keep UI data fresh
    fn start_background_updates(&self) {
        // Skip if we don't have a runtime
        let Some(runtime) = &self.background_runtime else {
            warn!("Cannot start background updates: no runtime available");
            return;
        };
        
        // Clone necessary components for the background thread
        let sidebar = self.sidebar.clone();
        let feed_repository = self.feed_repository.clone();
        let category_repository = self.category_repository.clone();
        let is_loading = self.is_loading.clone();
        let is_shutting_down = self.is_shutting_down.clone();
        
        // Spawn a thread to run the background tokio runtime
        thread::spawn(move || {
            info!("Starting background data update thread");
            
            runtime.block_on(async {
                loop {
                    // Check if we should shut down
                    if let Ok(is_shutting_down) = is_shutting_down.lock() {
                        if *is_shutting_down {
                            break;
                        }
                    }
                    
                    // Check if sidebar needs update
                    let needs_update = {
                        if let Ok(sidebar_guard) = sidebar.lock() {
                            if let Some(sidebar) = &*sidebar_guard {
                                sidebar.needs_data_loading()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };
                    
                    // Update sidebar data if needed
                    if needs_update {
                        // Set loading flag
                        if let Ok(mut is_loading) = is_loading.lock() {
                            *is_loading = true;
                        }
                        
                        // Create a temporary sidebar with the same repositories
                        let mut temp_sidebar = Sidebar::new(
                            feed_repository.clone(),
                            category_repository.clone(),
                        );
                        
                        // Copy the request state from the main sidebar
                        if let Ok(sidebar_guard) = sidebar.lock() {
                            if let Some(main_sidebar) = &*sidebar_guard {
                                temp_sidebar.state = main_sidebar.state.clone();
                            }
                        }
                        
                        // Update the data
                        match temp_sidebar.update_data_async().await {
                            Ok(_) => {
                                // Copy the updated data back to the main sidebar
                                if let Ok(mut sidebar_guard) = sidebar.lock() {
                                    if let Some(main_sidebar) = &mut *sidebar_guard {
                                        main_sidebar.data = temp_sidebar.data;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to update sidebar data: {}", e);
                            }
                        }
                        
                        // Clear loading flag
                        if let Ok(mut is_loading) = is_loading.lock() {
                            *is_loading = false;
                        }
                    }
                    
                    // Small delay to prevent CPU hogging
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            });
            
            info!("Background data update thread terminated");
        });
    }
    
    // Signal all background threads to shut down
    pub fn shutdown(&self) {
        info!("Shutting down AppContext");
        if let Ok(mut is_shutting_down) = self.is_shutting_down.lock() {
            *is_shutting_down = true;
        }
    }
}