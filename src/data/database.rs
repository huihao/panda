use std::path::Path;
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OpenFlags};
use std::sync::Arc;

pub fn init_database(db_path: &Path) -> Result<Arc<Pool<SqliteConnectionManager>>> {
    let manager = SqliteConnectionManager::file(db_path)
        .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE);
    
    let pool = Pool::new(manager)?;
    
    // Initialize database schema
    let conn = pool.get()?;
    conn.execute_batch(include_str!("../../data/schema.sql"))?;
    
    Ok(Arc::new(pool))
}

pub fn get_connection_pool(db_path: &Path) -> Result<Arc<Pool<SqliteConnectionManager>>> {
    let manager = SqliteConnectionManager::file(db_path)
        .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE);
    let pool = Pool::new(manager)?;
    Ok(Arc::new(pool))
}