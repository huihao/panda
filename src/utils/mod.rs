use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

pub fn current_timestamp() -> DateTime<Utc> {
    Utc::now()
}

pub fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Ensures that the directory for the given file path exists
///
/// This function extracts the directory part of a given file path
/// and creates it if it doesn't exist.
///
/// # Arguments
/// * `file_path` - The path to the file including the filename
///
/// # Returns
/// * `Result<()>` - Ok if the directory exists or was created successfully
pub fn ensure_directory_exists(file_path: &str) -> Result<()> {
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).with_context(|| 
                format!("Failed to create directory: {}", parent.display())
            )?;
        }
    }
    Ok(())
}