use std::sync::Arc;
use anyhow::Result;
use egui::{Ui, Window, TextEdit, DragValue};
use log::error;

use crate::services::SyncService;
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
    pub sync_interval: u32,
    /// Whether to automatically mark articles as read when opened
    pub auto_mark_read: bool,
    /// Whether to show article previews in the list
    pub show_previews: bool,
    /// Maximum number of articles to keep per feed
    pub max_articles_per_feed: u32,
    /// Maximum age of articles to keep (in days)
    pub max_article_age_days: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            sync_interval: 60,
            auto_mark_read: true,
            show_previews: true,
            max_articles_per_feed: 1000,
            max_article_age_days: 30,
        }
    }
}

/// Settings dialog component
pub struct SettingsDialog {
    sync_service: Arc<SyncService>,
    colors: AppColors,
    is_open: bool,
    sync_interval: u32,
    is_modified: bool,
}

impl SettingsDialog {
    /// Creates a new settings dialog
    pub fn new(sync_service: Arc<SyncService>) -> Self {
        Self {
            sync_service,
            colors: AppColors::default(),
            is_open: false,
            sync_interval: 60,
            is_modified: false,
        }
    }
    
    /// Shows the settings dialog
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        if !self.is_open {
            return Ok(());
        }
        
        Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .min_width(400.0)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    // Sync settings section
                    ui.group(|ui| {
                        ui.label("Synchronization");
                        ui.add_space(DEFAULT_PADDING);
                        
                        ui.horizontal(|ui| {
                            ui.label("Update interval:");
                            if ui.add(
                                DragValue::new(&mut self.sync_interval)
                                    .clamp_range(5..=1440)
                                    .suffix(" min")
                                    .speed(5)
                            ).changed() {
                                self.is_modified = true;
                            }
                        });
                        
                        if ui.button("Sync Now").clicked() {
                            if let Err(e) = self.sync_service.sync_all_feeds() {
                                error!("Failed to sync feeds: {}", e);
                            }
                        }
                    });
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Theme settings (placeholder for future customization)
                    ui.group(|ui| {
                        ui.label("Theme");
                        ui.add_space(DEFAULT_PADDING);
                        
                        // UI theme selection (will be implemented later)
                        ui.label("Theme selection coming soon...");
                    });
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Privacy settings
                    ui.group(|ui| {
                        ui.label("Privacy");
                        ui.add_space(DEFAULT_PADDING);
                        
                        ui.checkbox(&mut false, "Enable automatic article cleanup (coming soon)");
                        ui.add_enabled_ui(false, |ui| {
                            ui.add(
                                DragValue::new(&mut 30)
                                    .clamp_range(1..=365)
                                    .suffix(" days")
                                    .prefix("Keep articles for ")
                            );
                        });
                    });
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Buttons
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.close();
                        }
                        
                        if ui.add_enabled(self.is_modified, egui::Button::new("Save")).clicked() {
                            if let Err(e) = self.save_settings() {
                                error!("Failed to save settings: {}", e);
                            } else {
                                self.close();
                            }
                        }
                    });
                });
            });
        
        Ok(())
    }
    
    /// Opens the settings dialog
    pub fn open(&mut self) {
        self.is_open = true;
        self.sync_interval = self.sync_service.get_sync_interval();
        self.is_modified = false;
    }
    
    /// Closes the settings dialog
    pub fn close(&mut self) {
        self.is_open = false;
        self.is_modified = false;
    }
    
    /// Saves the current settings
    fn save_settings(&mut self) -> Result<()> {
        self.sync_service.set_sync_interval(self.sync_interval)?;
        self.is_modified = false;
        Ok(())
    }
}