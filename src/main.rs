mod data;
mod models;
mod services;
mod base;
mod utils;

use std::sync::Arc;
use anyhow::{Result, anyhow};
use log::{info, warn};
use url::Url;
use data::Database;
use services::{ArticleService, RssService, WebViewService};

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
    
    // Initialize services
    let article_service = ArticleService::new(article_repository.clone());
    let rss_service = RssService::new(feed_repository.clone(), article_repository.clone());
    let webview_service = WebViewService::new(article_service.clone());
    
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
    
    // Get available articles after syncing
    let articles = article_repository.get_all_articles()?;
    
    // Handle the case where no articles are available
    if articles.is_empty() {
        warn!("No articles found after syncing feeds. Nothing to display.");
        return Ok(());
    }
    
    // Start webview with the first available article
    info!("Starting webview service with first available article...");
    webview_service.create_article_view(&articles[0].id.0)?;
    
    Ok(())
}
