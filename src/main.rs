mod data;
mod models;
mod services;
mod base;
mod utils;
mod ui;

use std::sync::Arc;
use std::thread;
use anyhow::{Result, anyhow};
use log::{info, warn};
use url::Url;
use tokio::sync::mpsc;
use tokio::runtime::Runtime;
// Use the re-exported Database to follow Interface Segregation Principle
use crate::data::Database;
use crate::data::repositories::*; // Import repositories directly
use services::{RssService, WebViewService, SyncService, OpmlService};
use ui::views::main::MainView;
use eframe::egui;
use egui::ViewportBuilder;

// Main function with no tokio::main attribute - we'll create the runtime manually
fn main() -> Result<()> {
    // Set up logging
    env_logger::init();
    info!("Starting Panda RSS reader...");
    
    // Create a new channel for communicating between the async thread and UI thread
    let (tx, rx) = mpsc::channel(100);
    
    // Start the async operations in a separate thread
    let async_thread = thread::spawn(move || {
        // Create a new runtime for this thread
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        
        // Use runtime.block_on to run our async initialization
        runtime.block_on(async {
            init_async_services(tx).await
        })
    });
    
    // Initialize UI on the main thread (without being inside a Tokio runtime)
    let main_view = init_ui(rx)?;
    
    // Run the UI on the main thread
    run_ui(main_view)?;
    
    // Wait for the async thread to complete if necessary
    if let Err(e) = async_thread.join() {
        warn!("Async thread panicked: {:?}", e);
    }
    
    Ok(())
}

/// Initialize all async services and return an AppContext
async fn init_async_services(tx: mpsc::Sender<()>) -> Result<()> {
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
    
    // Initialize services - Fix the argument order to match the service constructor signature
    // Following Liskov Substitution Principle by ensuring proper contract adherence
    let rss_service = Arc::new(RssService::new(
        article_repository.clone(),
        feed_repository.clone(),
        category_repository.clone(),
        tag_repository.clone(),
    ));
    let webview_service = Arc::new(WebViewService::new());
    let sync_service = Arc::new(SyncService::new(rss_service.clone()));
    let opml_service = Arc::new(OpmlService::new(rss_service.clone()));
    
    // Check if there are any feeds in the database
    let feeds = feed_repository.get_all_feeds().await?;
    
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
    
    // Signal that initialization is complete
    let _ = tx.send(()).await;
    
    Ok(())
}

/// Initialize the UI components
fn init_ui(rx: mpsc::Receiver<()>) -> Result<MainView> {
    // Create database connection for UI thread
    let database_path = "data/panda.db";
    let database = Database::new(database_path)?;
    
    // Get repository instances for UI
    let article_repository = database.get_article_repository();
    let category_repository = database.get_category_repository();
    let feed_repository = database.get_feed_repository();
    let tag_repository = database.get_tag_repository();
    
    // Create an AppContext instance with the repositories
    // The new constructor only requires repositories
    let app_context: ui::AppContext = ui::AppContext::new(
        article_repository,
        category_repository,
        feed_repository,
        tag_repository,
    );
    
    // Create the main view
    let main_view = MainView::new(app_context);
    
    Ok(main_view)
}

/// Runs the egui-based UI with the given MainView
/// This function is now completely separate from any Tokio runtime
fn run_ui(main_view: MainView) -> Result<()> {
    // Create and run the main view with updated options
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Panda RSS Reader"),
        ..Default::default()
    };
    
    // Use a separate Result handling for the eframe call to properly convert errors
    match eframe::run_native(
        // The window title is now set in the viewport builder, so we pass an empty string here
        "",
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
