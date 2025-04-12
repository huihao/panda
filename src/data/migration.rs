use anyhow::{Result, Context};
use log::{info, warn, debug};
use rusqlite::{Connection, Error as SqliteError};

/// Database migration manager that handles schema updates
/// 
/// This follows the Single Responsibility Principle by focusing solely on database migrations
pub struct MigrationManager<'a> {
    connection: &'a Connection,
}

impl<'a> MigrationManager<'a> {
    /// Creates a new migration manager
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }
    
    /// Run all necessary migrations to update the database schema
    pub fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations");
        
        // Create migrations table if it doesn't exist
        self.create_migrations_table()?;
        
        // Run each migration if needed
        self.migrate_add_site_url_to_feeds()?;
        self.migrate_add_last_fetched_at_to_feeds()?;
        self.migrate_add_next_fetch_at_to_feeds()?;
        
        info!("Database migrations completed successfully");
        Ok(())
    }
    
    /// Creates the migrations table to track which migrations have been applied
    fn create_migrations_table(&self) -> Result<()> {
        debug!("Creating migrations table if it doesn't exist");
        
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL
            )",
            [],
        ).context("Failed to create migrations table")?;
        
        Ok(())
    }
    
    /// Checks if a migration has been applied
    fn is_migration_applied(&self, name: &str) -> Result<bool> {
        let count: i64 = self.connection
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE name = ?",
                [name],
                |row| row.get(0),
            )
            .context("Failed to check if migration has been applied")?;
        
        Ok(count > 0)
    }
    
    /// Records that a migration has been applied
    fn record_migration(&self, name: &str) -> Result<()> {
        debug!("Recording migration '{}' as applied", name);
        
        self.connection
            .execute(
                "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
                [name],
            )
            .context("Failed to record migration")?;
        
        Ok(())
    }
    
    /// Migration: Add site_url column to feeds table
    /// 
    /// Following the Open/Closed Principle, this method gracefully handles various
    /// scenarios without requiring modification to the core migration process.
    fn migrate_add_site_url_to_feeds(&self) -> Result<()> {
        const MIGRATION_NAME: &str = "add_site_url_to_feeds";
        
        // Check if this migration has already been applied in our migrations table
        if self.is_migration_applied(MIGRATION_NAME)? {
            debug!("Migration '{}' already recorded as applied, skipping", MIGRATION_NAME);
            return Ok(());
        }
        
        info!("Running migration: {}", MIGRATION_NAME);
        
        // First check if the feeds table exists
        let table_exists = match self.connection.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='feeds'",
            [],
            |_| Ok(true),
        ) {
            Ok(_) => true,
            Err(SqliteError::QueryReturnedNoRows) => false,
            Err(e) => return Err(e).context("Failed to check if feeds table exists"),
        };
        
        if !table_exists {
            // If the table doesn't exist yet, this migration will be applied when the table is created
            // via the schema.sql file, so we can just record it as complete
            debug!("Feeds table does not exist yet - migration will be applied through schema creation");
            self.record_migration(MIGRATION_NAME)?;
            return Ok(());
        }
        
        // Check if the column already exists (to make the migration idempotent)
        let column_exists = match self.connection.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name='site_url'",
            [],
            |_| Ok(true),
        ) {
            Ok(_) => true,
            Err(SqliteError::QueryReturnedNoRows) => false,
            Err(e) => return Err(e).context("Failed to check if site_url column exists"),
        };
        
        if !column_exists {
            debug!("Column 'site_url' does not exist, attempting to add it");
            // Try to add the column, but handle the case where it might have been added in parallel
            match self.connection.execute(
                "ALTER TABLE feeds ADD COLUMN site_url TEXT",
                [],
            ) {
                Ok(_) => info!("Successfully added site_url column to feeds table"),
                // Handle duplicate column error gracefully
                Err(e) if e.to_string().contains("duplicate column name") => {
                    info!("Column 'site_url' already exists (concurrent addition detected)");
                },
                Err(e) => return Err(e).context("Failed to add site_url column to feeds table"),
            }
        } else {
            info!("Column 'site_url' already exists in feeds table");
        }
        
        // Record the migration as complete regardless of how the column got there
        self.record_migration(MIGRATION_NAME)?;
        
        Ok(())
    }

    /// Migration: Add last_fetched_at column to feeds table
    /// 
    /// Following the Single Responsibility Principle, this method handles only
    /// the specific task of adding the last_fetched_at column to the feeds table.
    fn migrate_add_last_fetched_at_to_feeds(&self) -> Result<()> {
        const MIGRATION_NAME: &str = "add_last_fetched_at_to_feeds";
        
        // Check if this migration has already been applied in our migrations table
        if self.is_migration_applied(MIGRATION_NAME)? {
            debug!("Migration '{}' already recorded as applied, skipping", MIGRATION_NAME);
            return Ok(());
        }
        
        info!("Running migration: {}", MIGRATION_NAME);
        
        // First check if the feeds table exists
        let table_exists = match self.connection.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='feeds'",
            [],
            |_| Ok(true),
        ) {
            Ok(_) => true,
            Err(SqliteError::QueryReturnedNoRows) => false,
            Err(e) => return Err(e).context("Failed to check if feeds table exists"),
        };
        
        if !table_exists {
            // If the table doesn't exist yet, this migration will be applied when the table is created
            // via the schema.sql file, so we can just record it as complete
            debug!("Feeds table does not exist yet - migration will be applied through schema creation");
            self.record_migration(MIGRATION_NAME)?;
            return Ok(());
        }
        
        // Check if the column already exists (to make the migration idempotent)
        let column_exists = match self.connection.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name='last_fetched_at'",
            [],
            |_| Ok(true),
        ) {
            Ok(_) => true,
            Err(SqliteError::QueryReturnedNoRows) => false,
            Err(e) => return Err(e).context("Failed to check if last_fetched_at column exists"),
        };
        
        if !column_exists {
            debug!("Column 'last_fetched_at' does not exist, attempting to add it");
            // Try to add the column, but handle the case where it might have been added in parallel
            match self.connection.execute(
                "ALTER TABLE feeds ADD COLUMN last_fetched_at TEXT",
                [],
            ) {
                Ok(_) => info!("Successfully added last_fetched_at column to feeds table"),
                // Handle duplicate column error gracefully
                Err(e) if e.to_string().contains("duplicate column name") => {
                    info!("Column 'last_fetched_at' already exists (concurrent addition detected)");
                },
                Err(e) => return Err(e).context("Failed to add last_fetched_at column to feeds table"),
            }
        } else {
            info!("Column 'last_fetched_at' already exists in feeds table");
        }
        
        // Record the migration as complete regardless of how the column got there
        self.record_migration(MIGRATION_NAME)?;
        
        Ok(())
    }

    /// Migration: Add next_fetch_at column to feeds table
    /// 
    /// Following the Single Responsibility Principle, this method handles only
    /// the specific task of adding the next_fetch_at column to the feeds table.
    fn migrate_add_next_fetch_at_to_feeds(&self) -> Result<()> {
        const MIGRATION_NAME: &str = "add_next_fetch_at_to_feeds";
        
        // Check if this migration has already been applied in our migrations table
        if self.is_migration_applied(MIGRATION_NAME)? {
            debug!("Migration '{}' already recorded as applied, skipping", MIGRATION_NAME);
            return Ok(());
        }
        
        info!("Running migration: {}", MIGRATION_NAME);
        
        // First check if the feeds table exists
        let table_exists = match self.connection.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='feeds'",
            [],
            |_| Ok(true),
        ) {
            Ok(_) => true,
            Err(SqliteError::QueryReturnedNoRows) => false,
            Err(e) => return Err(e).context("Failed to check if feeds table exists"),
        };
        
        if !table_exists {
            // If the table doesn't exist yet, this migration will be applied when the table is created
            // via the schema.sql file, so we can just record it as complete
            debug!("Feeds table does not exist yet - migration will be applied through schema creation");
            self.record_migration(MIGRATION_NAME)?;
            return Ok(());
        }
        
        // Check if the column already exists (to make the migration idempotent)
        let column_exists = match self.connection.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name='next_fetch_at'",
            [],
            |_| Ok(true),
        ) {
            Ok(_) => true,
            Err(SqliteError::QueryReturnedNoRows) => false,
            Err(e) => return Err(e).context("Failed to check if next_fetch_at column exists"),
        };
        
        if !column_exists {
            debug!("Column 'next_fetch_at' does not exist, attempting to add it");
            // Try to add the column, but handle the case where it might have been added in parallel
            match self.connection.execute(
                "ALTER TABLE feeds ADD COLUMN next_fetch_at TEXT",
                [],
            ) {
                Ok(_) => info!("Successfully added next_fetch_at column to feeds table"),
                // Handle duplicate column error gracefully
                Err(e) if e.to_string().contains("duplicate column name") => {
                    info!("Column 'next_fetch_at' already exists (concurrent addition detected)");
                },
                Err(e) => return Err(e).context("Failed to add next_fetch_at column to feeds table"),
            }
        } else {
            info!("Column 'next_fetch_at' already exists in feeds table");
        }
        
        // Record the migration as complete regardless of how the column got there
        self.record_migration(MIGRATION_NAME)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    #[test]
    fn test_create_migrations_table() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        let manager = MigrationManager::new(&conn);
        
        manager.create_migrations_table()?;
        
        // Verify the table exists
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='migrations'",
            [],
            |row| row.get(0),
        )?;
        
        assert_eq!(count, 1, "Migrations table should exist");
        Ok(())
    }
    
    #[test]
    fn test_migration_tracking() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        let manager = MigrationManager::new(&conn);
        
        manager.create_migrations_table()?;
        
        let test_migration = "test_migration";
        
        // Initially, migration should not be applied
        assert!(!manager.is_migration_applied(test_migration)?, "Migration should not be applied initially");
        
        // Record the migration
        manager.record_migration(test_migration)?;
        
        // Now it should be applied
        assert!(manager.is_migration_applied(test_migration)?, "Migration should be applied after recording");
        
        Ok(())
    }
    
    #[test]
    fn test_add_site_url_migration() -> Result<()> {
        // Create in-memory database with test feeds table (without site_url)
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE feeds (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL
            )", 
            [],
        )?;
        
        let manager = MigrationManager::new(&conn);
        manager.create_migrations_table()?;
        
        // Run the migration
        manager.migrate_add_site_url_to_feeds()?;
        
        // Verify the site_url column exists
        let has_column = conn.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name = 'site_url'",
            [],
            |_| Ok(true),
        ).is_ok();
        
        assert!(has_column, "site_url column should exist after migration");
        
        // Running the migration again should be a no-op
        manager.migrate_add_site_url_to_feeds()?;
        
        Ok(())
    }
    
    #[test]
    fn test_add_last_fetched_at_migration() -> Result<()> {
        // Create in-memory database with test feeds table (without last_fetched_at)
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE feeds (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )", 
            [],
        )?;
        
        let manager = MigrationManager::new(&conn);
        manager.create_migrations_table()?;
        
        // Run the migration
        manager.migrate_add_last_fetched_at_to_feeds()?;
        
        // Verify the last_fetched_at column exists
        let has_column = conn.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name = 'last_fetched_at'",
            [],
            |_| Ok(true),
        ).is_ok();
        
        assert!(has_column, "last_fetched_at column should exist after migration");
        
        // Running the migration again should be a no-op and not error
        manager.migrate_add_last_fetched_at_to_feeds()?;
        
        Ok(())
    }
    
    #[test]
    fn test_add_next_fetch_at_migration() -> Result<()> {
        // Create in-memory database with test feeds table (without next_fetch_at)
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE feeds (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )", 
            [],
        )?;
        
        let manager = MigrationManager::new(&conn);
        manager.create_migrations_table()?;
        
        // Run the migration
        manager.migrate_add_next_fetch_at_to_feeds()?;
        
        // Verify the next_fetch_at column exists
        let has_column = conn.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name = 'next_fetch_at'",
            [],
            |_| Ok(true),
        ).is_ok();
        
        assert!(has_column, "next_fetch_at column should exist after migration");
        
        // Running the migration again should be a no-op and not error
        manager.migrate_add_next_fetch_at_to_feeds()?;
        
        Ok(())
    }
    
    #[test]
    fn test_all_migrations_idempotent() -> Result<()> {
        // Create in-memory database 
        let conn = Connection::open_in_memory()?;
        let manager = MigrationManager::new(&conn);
        
        // Create basic tables structure
        conn.execute_batch("
            CREATE TABLE feeds (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            
            CREATE TABLE migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL
            );
        ")?;
        
        // Run migrations twice
        manager.run_migrations()?;
        
        // Running again should be a no-op
        manager.run_migrations()?;
        
        // Verify both columns exist
        let has_site_url = conn.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name = 'site_url'",
            [],
            |_| Ok(true),
        ).is_ok();
        
        let has_last_fetched_at = conn.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name = 'last_fetched_at'",
            [],
            |_| Ok(true),
        ).is_ok();
        
        let has_next_fetch_at = conn.query_row(
            "SELECT 1 FROM pragma_table_info('feeds') WHERE name = 'next_fetch_at'",
            [],
            |_| Ok(true),
        ).is_ok();
        
        assert!(has_site_url, "site_url column should exist");
        assert!(has_last_fetched_at, "last_fetched_at column should exist");
        assert!(has_next_fetch_at, "next_fetch_at column should exist");
        
        Ok(())
    }
}