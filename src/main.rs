mod data;
mod models;
mod services;
mod base;
mod utils;
mod ui;

use std::sync::Arc;
use anyhow::{Result, anyhow};
use log::{info, warn};
use url::Url;
use data::Database;
use services::{RssService, WebViewService, SyncService, OpmlService};
use ui::views::main::MainView;
use eframe::egui;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    env_logger::init();
    info!("Starting Panda RSS reader...");
    
    // Initialize database with a path to the SQLite database file
    let database_path = "data/panda.db";
    
    // Ensure the data directory exists before trying to open the database file
    info!("Ensuring database directory exists...");
    std::fs::create_dir_all("data")?;
    
    // Create database connection
    info!("Initializing database...");
    let database = Database::new(database_path)?;
    
    // Obtain repository instances from the database
    let article_repository = database.get_article_repository();
    let feed_repository = database.get_feed_repository();
    let category_repository = database.get_category_repository();
    let tag_repository = database.get_tag_repository();
    
    // Initialize services
    let rss_service = Arc::new(RssService::new(
        feed_repository.clone(),
        article_repository.clone(),
        category_repository.clone(),
        tag_repository.clone(),
    ));
    let webview_service = Arc::new(WebViewService::new(article_repository.clone()));
    let sync_service = Arc::new(SyncService::new(rss_service.clone()));
    let opml_service = Arc::new(OpmlService::new(rss_service.clone()));
    
    // Check if there are any feeds in the database
    let feeds = feed_repository.get_all_feeds()?;
    
    // Add a default feed if none exists
    if feeds.is_empty() {
        info!("No feeds found. Adding default feeds...");
        let default_feeds = [
            ("Hacker News", "https://news.ycombinator.com/rss"),
            ("BBC News", "http://feeds.bbci.co.uk/news/rss.xml"),
            ("TechCrunch", "https://techcrunch.com/feed/"),
        ];
        
        for (title, url_str) in default_feeds {
            match Url::parse(url_str) {
                Ok(url) => {
                    info!("Adding default feed: {}", title);
                    if let Err(e) = rss_service.add_feed(&url.to_string()).await {
                        warn!("Failed to add default feed {}: {}", title, e);
                    }
                }
                Err(e) => {
                    warn!("Invalid URL for default feed {}: {}", title, e);
                }
            }
        }
    }
    
    // Fetch feeds
    info!("Syncing feeds...");
    if let Err(e) = rss_service.sync_all_feeds().await {
        warn!("Failed to sync all feeds: {}", e);
    }
    
    // Prepare the main view and UI configuration
    let main_view = MainView::new(
        feed_repository,
        category_repository,
        article_repository,
        rss_service,
        webview_service,
        sync_service,
    );
    
    // Run the UI in a separate thread or context to avoid Send issues
    // This follows the Dependency Inversion Principle by isolating the UI framework
    // from the async runtime
    run_ui(main_view)?;
    
    Ok(())
}

/// Runs the egui-based UI with the given MainView
/// This separates the UI thread from the tokio async runtime to avoid Send trait issues
fn run_ui(main_view: MainView) -> Result<()> {
    // Create and run the main view
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        ..Default::default()
    };
    
    // Use a separate Result handling for the eframe call to properly convert errors
    // The closure now correctly returns a Result<Box<dyn eframe::App>, Box<dyn Error>>
    match eframe::run_native(
        "Panda RSS Reader",
        options,
        Box::new(|_cc| -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            // Wrap the main_view in Ok to satisfy the Result return type
            Ok(Box::new(main_view))
        }),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error running UI: {}", e)),
    }
}
