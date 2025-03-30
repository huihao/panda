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
use services::{ArticleService, RssService, WebViewService, SyncService};
use ui::views::main::MainView;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    env_logger::init();
    info!("Starting Panda RSS reader...");
    
    // Initialize database with a path to the SQLite database file
    let database_path = "data/panda.db";
    
    // Ensure the data directory exists before trying to open the database file
    info!("Ensuring database directory exists...");
    utils::ensure_directory_exists(database_path)?;
    
    // Create database connection
    info!("Initializing database...");
    let database = Database::new(database_path)?;
    
    // Obtain repository instances from the database
    let article_repository = database.article_repository();
    let feed_repository = database.feed_repository();
    let category_repository = database.category_repository();
    
    // Initialize services
    let article_service = ArticleService::new(article_repository.clone());
    let rss_service = RssService::new(feed_repository.clone(), article_repository.clone());
    let webview_service = WebViewService::new(article_service.clone());
    let sync_service = SyncService::new(rss_service.clone());
    
    // Check if there are any feeds in the database
    let feeds = feed_repository.get_all_feeds()?;
    
    // Add a default feed if none exists (following OCP by extending functionality)
    if feeds.is_empty() {
        info!("No feeds found. Adding a default feed...");
        let default_feed_url = Url::parse("https://news.ycombinator.com/rss")?;
        rss_service.add_feed(default_feed_url).await?;
    }
    
    // Fetch feeds
    info!("Syncing feeds...");
    rss_service.fetch_all_feeds().await?;
    
    // Create and run the main view
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        ..Default::default()
    };
    
    let main_view = MainView::new(
        feed_repository,
        category_repository,
        article_repository,
        rss_service,
        webview_service,
        sync_service,
    );
    
    eframe::run_native(
        "Panda RSS Reader",
        options,
        Box::new(|_cc| Box::new(main_view)),
    )?;
    
    Ok(())
}
