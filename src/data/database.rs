use std::path::Path;
use anyhow::{Result, anyhow, Context};
use std::sync::{Arc, Mutex};
use rusqlite::{Connection, OpenFlags};
use log::{info, warn, debug};

// Import the repository traits from the base module
use crate::base::repository::{
    ArticleRepository,
    FeedRepository, 
    CategoryRepository,
    TagRepository
};

// Import the re-exported repository implementations directly
// This follows the Dependency Inversion Principle by depending on abstractions
use crate::data::repositories::{
    SqliteArticleRepository,
    SqliteFeedRepository,
    SqliteCategoryRepository,
    SqliteTagRepository
};

// Import migration manager for schema updates
use crate::data::migration::MigrationManager;

/// A simple connection pool implementation to avoid dependency conflicts
/// with external connection pool libraries
pub struct ConnectionPool {
    connections: Mutex<Vec<Connection>>,
    db_path: String,
    max_connections: usize,
}

impl ConnectionPool {
    /// Create a new connection pool with the specified maximum number of connections
    pub fn new(db_path: &str, max_connections: usize) -> Result<Self> {
        let mut connections = Vec::with_capacity(max_connections);
        
        // Create initial connection
        let initial_connection = Self::create_connection(db_path)?;
        
        // Initialize schema on the first connection
        initial_connection.execute_batch(include_str!("../../data/schema.sql"))?;
        
        // Run migrations to ensure the database schema is up-to-date using the new MigrationManager
        let migration_manager = MigrationManager::new(&initial_connection);
        migration_manager.run_migrations()
            .context("Failed to run database migrations")?;
        
        connections.push(initial_connection);
        
        Ok(Self {
            connections: Mutex::new(connections),
            db_path: db_path.to_string(),
            max_connections,
        })
    }
    
    /// Create a new SQLite connection
    fn create_connection(db_path: &str) -> Result<Connection> {
        let db_path = Path::new(db_path);
        // Make sure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Open connection with ability to create the database if it doesn't exist
        let connection = Connection::open_with_flags(
            db_path, 
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        )?;
        
        // Enable foreign keys
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        
        Ok(connection)
    }
    
    /// Get a connection from the pool or create a new one if needed
    pub fn get(&self) -> Result<PooledConnection> {
        let mut connections = self.connections.lock().map_err(|_| anyhow!("Failed to lock connection pool"))?;
        
        if let Some(connection) = connections.pop() {
            return Ok(PooledConnection {
                connection: Some(connection),
                pool: self,
            });
        }
        
        // If we've reached max connections, return an error
        if connections.len() >= self.max_connections {
            return Err(anyhow!("Connection pool exhausted"));
        }
        
        // Create a new connection
        let connection = Self::create_connection(&self.db_path)?;
        
        Ok(PooledConnection {
            connection: Some(connection),
            pool: self,
        })
    }
    
    /// Return a connection to the pool
    fn return_connection(&self, connection: Connection) -> Result<()> {
        let mut connections = self.connections.lock().map_err(|_| anyhow!("Failed to lock connection pool"))?;
        connections.push(connection);
        Ok(())
    }
}

/// A connection that returns to the pool when dropped
pub struct PooledConnection<'a> {
    connection: Option<Connection>,
    pool: &'a ConnectionPool,
}

impl<'a> std::ops::Deref for PooledConnection<'a> {
    type Target = Connection;
    
    fn deref(&self) -> &Self::Target {
        self.connection.as_ref().expect("Connection was taken")
    }
}

impl<'a> std::ops::DerefMut for PooledConnection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection.as_mut().expect("Connection was taken")
    }
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            // Ignore errors when returning connections to the pool
            let _ = self.pool.return_connection(conn);
        }
    }
}

/// Database is the main entry point for database operations
/// It follows the Repository pattern and provides access to all repositories
pub struct Database {
    connection_pool: Arc<ConnectionPool>,
}

impl Database {
    /// Create a new Database instance with the given SQLite database path
    /// Uses a default connection pool size of 5
    pub fn new(db_path: &str) -> Result<Self> {
        Self::with_pool_size(db_path, 5)
    }
    
    /// Create a new Database instance with a specified connection pool size
    pub fn with_pool_size(db_path: &str, pool_size: usize) -> Result<Self> {
        let connection_pool = ConnectionPool::new(db_path, pool_size)?;
        
        Ok(Self {
            connection_pool: Arc::new(connection_pool),
        })
    }
    
    /// Get the article repository implementation
    pub fn get_article_repository(&self) -> Arc<dyn ArticleRepository> {
        Arc::new(SqliteArticleRepository::new(self.connection_pool.clone()))
    }
    
    /// Get the feed repository implementation
    pub fn get_feed_repository(&self) -> Arc<dyn FeedRepository> {
        Arc::new(SqliteFeedRepository::new(self.connection_pool.clone()))
    }
    
    /// Get the category repository implementation
    pub fn get_category_repository(&self) -> Arc<dyn CategoryRepository> {
        Arc::new(SqliteCategoryRepository::new(self.connection_pool.clone()))
    }
    
    /// Get the tag repository implementation
    pub fn get_tag_repository(&self) -> Arc<dyn TagRepository> {
        Arc::new(SqliteTagRepository::new(self.connection_pool.clone()))
    }
}