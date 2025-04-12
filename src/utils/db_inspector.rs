use anyhow::{Result, Context};
use log::{info, warn, debug};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::collections::HashMap;

/// Database inspector for debugging schema issues
/// 
/// This follows the Single Responsibility Principle by focusing solely on database inspection
pub struct DbInspector {
    connection: Connection,
}

impl DbInspector {
    /// Create a new database inspector for the specified database file
    pub fn new(db_path: &str) -> Result<Self> {
        let db_path = Path::new(db_path);
        
        if !db_path.exists() {
            return Err(anyhow::anyhow!("Database file does not exist: {}", db_path.display()));
        }
        
        // Open connection in read-only mode to avoid modifying the database
        let connection = Connection::open_with_flags(
            db_path, 
            OpenFlags::SQLITE_OPEN_READ_ONLY
        ).context("Failed to open database connection")?;
        
        Ok(Self { connection })
    }
    
    /// Get a list of all tables in the database
    pub fn get_tables(&self) -> Result<Vec<String>> {
        let mut stmt = self.connection.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        ).context("Failed to prepare statement to list tables")?;
        
        let table_names = stmt.query_map([], |row| row.get(0))
            .context("Failed to query tables")?
            .collect::<std::result::Result<Vec<String>, _>>()
            .context("Failed to collect table names")?;
            
        Ok(table_names)
    }
    
    /// Check if a table exists in the database
    pub fn table_exists(&self, table_name: &str) -> Result<bool> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
            [table_name],
            |row| row.get(0)
        ).context("Failed to check if table exists")?;
        
        Ok(count > 0)
    }
    
    /// Get the schema for a specific table
    pub fn get_table_schema(&self, table_name: &str) -> Result<Vec<TableColumn>> {
        if !self.table_exists(table_name)? {
            return Err(anyhow::anyhow!("Table does not exist: {}", table_name));
        }
        
        let mut stmt = self.connection.prepare(
            "PRAGMA table_info(?)"
        ).context("Failed to prepare statement to get table schema")?;
        
        let columns = stmt.query_map([table_name], |row| {
            Ok(TableColumn {
                cid: row.get(0)?,
                name: row.get(1)?,
                column_type: row.get(2)?,
                notnull: row.get(3)?,
                default_value: row.get(4)?,
                pk: row.get(5)?,
            })
        }).context("Failed to query table schema")?
        .collect::<std::result::Result<Vec<TableColumn>, _>>()
        .context("Failed to collect table columns")?;
        
        Ok(columns)
    }
    
    /// Check if a column exists in a table
    pub fn column_exists(&self, table_name: &str, column_name: &str) -> Result<bool> {
        let columns = self.get_table_schema(table_name)?;
        
        for column in columns {
            if column.name == column_name {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Get row count for a table
    pub fn get_row_count(&self, table_name: &str) -> Result<i64> {
        if !self.table_exists(table_name)? {
            return Err(anyhow::anyhow!("Table does not exist: {}", table_name));
        }
        
        let count: i64 = self.connection.query_row(
            &format!("SELECT COUNT(*) FROM {}", table_name),
            [],
            |row| row.get(0)
        ).context("Failed to get row count")?;
        
        Ok(count)
    }
    
    /// Print a report of the database structure
    pub fn print_database_report(&self) -> Result<()> {
        info!("Database Structure Report");
        info!("=======================");
        
        let tables = self.get_tables()?;
        info!("Found {} tables", tables.len());
        
        for table_name in &tables {
            let row_count = self.get_row_count(table_name)?;
            let columns = self.get_table_schema(table_name)?;
            
            info!("Table: {} ({} rows)", table_name, row_count);
            
            for column in &columns {
                let null_str = if column.notnull > 0 { "NOT NULL" } else { "NULL" };
                let pk_str = if column.pk > 0 { "PRIMARY KEY" } else { "" };
                let default_str = match &column.default_value {
                    Some(val) => format!("DEFAULT '{}'", val),
                    None => "".to_string(),
                };
                
                info!("  - {}: {} {} {} {}", 
                    column.name, 
                    column.column_type, 
                    null_str, 
                    pk_str, 
                    default_str
                );
            }
            
            info!("");
        }
        
        info!("End of Database Report");
        
        Ok(())
    }
    
    /// Check for schema issues related to the feeds table
    pub fn check_feeds_table(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        
        // Check if feeds table exists
        if !self.table_exists("feeds")? {
            issues.push("Table 'feeds' does not exist".to_string());
            return Ok(issues);
        }
        
        // Check for site_url column
        if !self.column_exists("feeds", "site_url")? {
            issues.push("Column 'site_url' does not exist in table 'feeds'".to_string());
        }
        
        // Check other required columns
        let required_columns = vec!["id", "title", "url", "category_id", "status", "created_at", "updated_at"];
        
        for column in required_columns {
            if !self.column_exists("feeds", column)? {
                issues.push(format!("Required column '{}' does not exist in table 'feeds'", column));
            }
        }
        
        Ok(issues)
    }
}

/// Represents a column in a database table
#[derive(Debug, Clone)]
pub struct TableColumn {
    pub cid: i32,
    pub name: String,
    pub column_type: String,
    pub notnull: i32,
    pub default_value: Option<String>,
    pub pk: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::tempdir;
    use std::fs;
    use std::path::PathBuf;
    
    /// Create a test database with some tables
    fn create_test_db() -> (PathBuf, Connection) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let conn = Connection::open(&db_path).unwrap();
        
        conn.execute(
            "CREATE TABLE test_table (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                value INTEGER,
                created_at TEXT NOT NULL
            )",
            [],
        ).unwrap();
        
        conn.execute(
            "CREATE TABLE feeds (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        ).unwrap();
        
        // Keep the temporary directory alive
        std::mem::forget(dir);
        
        (db_path, conn)
    }
    
    #[test]
    fn test_get_tables() {
        let (db_path, _conn) = create_test_db();
        
        let inspector = DbInspector::new(db_path.to_str().unwrap()).unwrap();
        let tables = inspector.get_tables().unwrap();
        
        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"test_table".to_string()));
        assert!(tables.contains(&"feeds".to_string()));
    }
    
    #[test]
    fn test_table_exists() {
        let (db_path, _conn) = create_test_db();
        
        let inspector = DbInspector::new(db_path.to_str().unwrap()).unwrap();
        
        assert!(inspector.table_exists("test_table").unwrap());
        assert!(inspector.table_exists("feeds").unwrap());
        assert!(!inspector.table_exists("nonexistent").unwrap());
    }
    
    #[test]
    fn test_get_table_schema() {
        let (db_path, _conn) = create_test_db();
        
        let inspector = DbInspector::new(db_path.to_str().unwrap()).unwrap();
        let columns = inspector.get_table_schema("test_table").unwrap();
        
        assert_eq!(columns.len(), 4);
        
        // Check the primary key
        let pk_column = columns.iter().find(|c| c.pk > 0).unwrap();
        assert_eq!(pk_column.name, "id");
    }
    
    #[test]
    fn test_column_exists() {
        let (db_path, _conn) = create_test_db();
        
        let inspector = DbInspector::new(db_path.to_str().unwrap()).unwrap();
        
        assert!(inspector.column_exists("test_table", "id").unwrap());
        assert!(inspector.column_exists("test_table", "name").unwrap());
        assert!(!inspector.column_exists("test_table", "nonexistent").unwrap());
    }
    
    #[test]
    fn test_feeds_table_missing_site_url() {
        let (db_path, _conn) = create_test_db();
        
        let inspector = DbInspector::new(db_path.to_str().unwrap()).unwrap();
        let issues = inspector.check_feeds_table().unwrap();
        
        // The test database is missing the site_url column
        assert!(issues.iter().any(|issue| issue.contains("site_url")));
    }
}