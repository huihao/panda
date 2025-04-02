use egui::{Ui, Window, RichText, Context, DragValue, ViewportCommand};
use std::sync::Arc;
use anyhow::Result;

use crate::services::sync::SyncService;
use crate::ui::styles::AppColors;

pub struct SettingsDialog {
    sync_service: Arc<SyncService>,
    colors: AppColors,
    show: bool,
    sync_interval: i32,
    article_retention_days: i32,
}

impl SettingsDialog {
    pub fn new(
        sync_service: Arc<SyncService>,
        colors: AppColors,
    ) -> Self {
        Self {
            sync_service,
            colors,
            show: false,
            sync_interval: 60,
            article_retention_days: 30,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Result<()> {
        if !self.show {
            return Ok(());
        }

        // Create local mutable copies of the state we need to modify
        let mut sync_interval = self.sync_interval;
        let mut article_retention_days = self.article_retention_days;
        let colors = &self.colors; // Reference instead of clone
        
        // Step 1: Create the window but capture its response instead of chaining
        let window = Window::new("Settings")
            .open(&mut self.show)  // Use self.show directly here
            .resizable(false);
            
        // Step 2: Show the window and handle UI content
        let response = window.show(ctx, |ui| {
            // Pass individual fields instead of self (no show reference here)
            ui_content(ui, colors, &mut sync_interval, &mut article_retention_days)
        });

        // Step 3: Update struct fields with any changes from the UI
        let changed = self.sync_interval != sync_interval || 
                      self.article_retention_days != article_retention_days;
        
        self.sync_interval = sync_interval;
        self.article_retention_days = article_retention_days;
        
        // If the dialog was closed and values changed, save settings
        if !self.show && changed {
            if let Err(e) = self.save_settings() {
                eprintln!("Error saving settings: {}", e);
            }
        }

        Ok(())
    }

    fn save_settings(&self) -> Result<()> {
        // Here we would save the settings to persistent storage
        // For now, we just print them
        println!("Saving settings:");
        println!("Sync interval: {} minutes", self.sync_interval);
        println!("Article retention: {} days", self.article_retention_days);
        Ok(())
    }

    pub fn open(&mut self) {
        self.show = true;
    }

    pub fn close(&mut self) {
        self.show = false;
    }
}

// Define the UI rendering function as a free function instead of a method
fn ui_content(
    ui: &mut Ui, 
    colors: &AppColors, 
    sync_interval: &mut i32, 
    article_retention_days: &mut i32,
) -> Result<()> {
    ui.vertical(|ui| {
        ui.heading(RichText::new("Sync Settings").color(colors.text_highlight));
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Sync Interval (minutes):");
            ui.add(DragValue::new(sync_interval)
                .clamp_range(15..=1440));
        });

        ui.add_space(16.0);
        ui.heading(RichText::new("Article Settings").color(colors.text_highlight));
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Article Retention (days):");
            ui.add(DragValue::new(article_retention_days)
                .clamp_range(1..=365));
        });

        ui.add_space(16.0);

        // The save button sets a close flag in egui that will be handled by the window
        if ui.button("Save Changes").clicked() {
            // Return true to indicate we want to close the dialog
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    });

    Ok(())
}