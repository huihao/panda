use serde::{Serialize, Deserialize};
use egui::{Ui, Window, TextEdit, DragValue, Button, RichText, Color32};
use log::error;
use std::sync::Arc;
use anyhow::Result;

use crate::services::sync::SyncService;
use crate::ui::styles::{AppColors, DEFAULT_PADDING};

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Application theme
    pub theme: Theme,
    /// Feed synchronization interval in minutes
    pub sync_interval: i64,
    /// Whether to automatically mark articles as read when opened
    pub auto_mark_read: bool,
    /// Whether to show article previews in the list
    pub show_previews: bool,
    /// Maximum number of articles to keep per feed
    pub max_articles_per_feed: u32,
    /// Maximum age of articles to keep (in days)
    pub max_article_age_days: u32,
    /// Whether to automatically sync feeds
    pub auto_sync: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            sync_interval: 300, // 5 minutes in seconds
            auto_mark_read: true,
            show_previews: true,
            max_articles_per_feed: 1000,
            max_article_age_days: 30,
            auto_sync: true,
        }
    }
}

/// Component for managing application settings
pub struct SettingsDialog {
    sync_service: Arc<SyncService>,
    colors: AppColors,
    visible: bool,
    error_message: Option<String>,
}

impl SettingsDialog {
    /// Creates a new settings dialog
    pub fn new(sync_service: Arc<SyncService>, colors: AppColors) -> Self {
        Self {
            sync_service,
            colors,
            visible: false,
            error_message: None,
        }
    }
    
    /// Shows the settings dialog
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        if self.visible {
            Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Sync Settings").color(self.colors.text));
                        
                        if let Some(error) = &self.error_message {
                            ui.label(RichText::new(error).color(self.colors.error));
                        }
                        
                        ui.horizontal(|ui| {
                            if ui.button("Sync Now").clicked() {
                                if let Err(e) = self.sync_service.sync_all_feeds() {
                                    error!("Failed to sync feeds: {}", e);
                                    self.error_message = Some(e.to_string());
                                } else {
                                    self.error_message = None;
                                }
                            }
                            
                            if ui.button("Close").clicked() {
                                self.visible = false;
                                self.error_message = None;
                            }
                        });
                    });
                });
        }
        Ok(())
    }
}