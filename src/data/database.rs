use std::path::Path;
use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use std::sync::{Arc, Mutex};
use crate::data::repositories::{
    SqliteFeedRepository,
    SqliteArticleRepository,
    SqliteTagRepository,
};
// Fix the import path to use the external crate reference
use crate::base::repository::FeedRepository;
use crate::base::repository::ArticleRepository;
use crate::base::repository::TagRepository;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

/// Manages database connections and repository instances
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Creates a new database manager
    pub fn new(path: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager)?;
        let db = Database { pool };
        db.init_database()?;
        Ok(db)
    }

    fn init_database(&self) -> Result<()> {
        let conn = self.pool.get()?;
        
        // Create feeds table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
                id TEXT PRIMARY KEY,
                url TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                icon_url TEXT,
                category_id TEXT,
                status TEXT NOT NULL,
                error_message TEXT,
                last_fetch INTEGER,
                next_fetch INTEGER,
                update_interval INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create articles table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS articles (
                id TEXT PRIMARY KEY,
                feed_id TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT UNIQUE NOT NULL,
                author TEXT,
                content TEXT NOT NULL,
                summary TEXT,
                published_at INTEGER NOT NULL,
                read_status INTEGER NOT NULL,
                is_favorite INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                thumbnail_url TEXT,
                FOREIGN KEY(feed_id) REFERENCES feeds(id)
            )",
            [],
        )?;

        // Create tags table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create article_tags table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS article_tags (
                article_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                PRIMARY KEY (article_id, tag_id),
                FOREIGN KEY(article_id) REFERENCES articles(id),
                FOREIGN KEY(tag_id) REFERENCES tags(id)
            )",
            [],
        )?;

        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_feeds_url ON feeds(url)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_articles_url ON articles(url)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_articles_published_at ON articles(published_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name)", [])?;

        Ok(())
    }

    /// Gets a database connection from the pool
    pub fn get_connection(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        Ok(self.pool.get()?)
    }

    /// Gets the database version number
    pub fn get_version(&self) -> Result<i32> {
        let version: i32 = self.pool.get()?.query_row(
            "SELECT value FROM pragma_user_version",
            [],
            |row| row.get(0),
        )?;
        Ok(version)
    }

    /// Sets the database version number
    pub fn set_version(&self, version: i32) -> Result<()> {
        self.pool.get()?.execute(
            "PRAGMA user_version = ?",
            [version],
        )?;
        Ok(())
    }

    /// Begins a transaction
    pub fn begin_transaction(&self) -> Result<()> {
        self.pool.get()?.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }

    /// Commits a transaction
    pub fn commit_transaction(&self) -> Result<()> {
        self.pool.get()?.execute("COMMIT", [])?;
        Ok(())
    }

    /// Rolls back a transaction
    pub fn rollback_transaction(&self) -> Result<()> {
        self.pool.get()?.execute("ROLLBACK", [])?;
        Ok(())
    }

    /// Creates a new feed repository
    pub fn get_feed_repository(&self) -> Box<dyn FeedRepository> {
        Box::new(SqliteFeedRepository::new(self.pool.clone()))
    }

    /// Creates a new article repository
    pub fn get_article_repository(&self) -> Box<dyn ArticleRepository> {
        Box::new(SqliteArticleRepository::new(self.pool.clone()))
    }

    /// Creates a new tag repository
    pub fn get_tag_repository(&self) -> Box<dyn TagRepository> {
        Box::new(SqliteTagRepository::new(self.pool.clone()))
    }

    pub fn article_repository(&self) -> Arc<dyn ArticleRepository> {
        Arc::new(SqliteArticleRepository::new(self.pool.clone()))
    }

    pub fn feed_repository(&self) -> Arc<dyn FeedRepository> {
        Arc::new(SqliteFeedRepository::new(self.pool.clone()))
    }

    pub fn tag_repository(&self) -> Arc<dyn TagRepository> {
        Arc::new(SqliteTagRepository::new(self.pool.clone()))
    }
}