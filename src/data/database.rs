use std::path::Path;
use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use std::sync::{Arc, Mutex};
use crate::data::repositories::{
    SqliteFeedRepository,
    SqliteArticleRepository,
    SqliteTagRepository,
    SqliteCategoryRepository,
};
// Fix the import path to use the external crate reference
use crate::base::repository::FeedRepository;
use crate::base::repository::ArticleRepository;
use crate::base::repository::TagRepository;
use crate::base::repository::CategoryRepository;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use log::error;

/// Manages database connections and repository instances
pub struct Database {
    /// The connection pool for database operations
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Creates a new database manager with the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path)
            .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE);
        let pool = Pool::new(manager)?;
        
        let db = Self { pool };
        db.init_database()?;
        
        Ok(db)
    }
    
    /// Initializes the database schema
    fn init_database(&self) -> Result<()> {
        let conn = self.pool.get()?;
        
        // Create categories table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                parent_id TEXT,
                is_expanded INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES categories(id)
            )",
            [],
        )?;
        
        // Create feeds table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
                id TEXT PRIMARY KEY,
                category_id TEXT,
                title TEXT NOT NULL,
                description TEXT,
                url TEXT NOT NULL,
                icon_url TEXT,
                site_url TEXT,
                language TEXT,
                last_fetched_at TEXT,
                last_fetch TEXT,
                next_fetch TEXT,
                update_interval INTEGER,
                is_enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (category_id) REFERENCES categories(id)
            )",
            [],
        )?;
        
        // Create articles table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS articles (
                id TEXT PRIMARY KEY,
                feed_id TEXT NOT NULL,
                category_id TEXT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                url TEXT NOT NULL,
                thumbnail_url TEXT,
                author TEXT,
                published_at TEXT NOT NULL,
                read_status TEXT NOT NULL,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                saved_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (feed_id) REFERENCES feeds(id),
                FOREIGN KEY (category_id) REFERENCES categories(id)
            )",
            [],
        )?;
        
        // Create tags table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create article_tags table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS article_tags (
                article_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (article_id, tag_id),
                FOREIGN KEY (article_id) REFERENCES articles(id),
                FOREIGN KEY (tag_id) REFERENCES tags(id)
            )",
            [],
        )?;
        
        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_feeds_url ON feeds(url)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_feeds_category ON feeds(category_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_feed ON articles(feed_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_category ON articles(category_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published_at)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_read_status ON articles(read_status)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_favorite ON articles(is_favorite)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_article_tags_article ON article_tags(article_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_article_tags_tag ON article_tags(tag_id)",
            [],
        )?;
        
        Ok(())
    }
    
    /// Begins a new transaction
    pub fn begin_transaction(&self) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }
    
    /// Commits the current transaction
    pub fn commit_transaction(&self) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute("COMMIT", [])?;
        Ok(())
    }
    
    /// Rolls back the current transaction
    pub fn rollback_transaction(&self) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute("ROLLBACK", [])?;
        Ok(())
    }
    
    /// Gets the article repository instance
    pub fn get_article_repository(&self) -> Arc<dyn ArticleRepository> {
        Arc::new(SqliteArticleRepository::new(self.pool.clone()))
    }
    
    /// Gets the feed repository instance
    pub fn get_feed_repository(&self) -> Arc<dyn FeedRepository> {
        Arc::new(SqliteFeedRepository::new(self.pool.clone()))
    }
    
    /// Gets the category repository instance
    pub fn get_category_repository(&self) -> Arc<dyn CategoryRepository> {
        Arc::new(SqliteCategoryRepository::new(self.pool.clone()))
    }
    
    /// Gets the tag repository instance
    pub fn get_tag_repository(&self) -> Arc<dyn TagRepository> {
        Arc::new(SqliteTagRepository::new(self.pool.clone()))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if let Err(e) = self.rollback_transaction() {
            error!("Failed to rollback transaction: {}", e);
        }
    }
}