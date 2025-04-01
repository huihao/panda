use egui::{Ui, Window, RichText};
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

        // 修复借用冲突：使用临时变量保存self.show的值
        let mut dialog_open = self.show;
        Window::new("Settings")
            .open(&mut dialog_open)
            .resizable(false)
            .show(ctx, |ui| {
                self.ui_content(ui)
            });
        
        // 更新show字段
        self.show = dialog_open;

        Ok(())
    }

    fn ui_content(&mut self, ui: &mut Ui) -> Result<()> {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Sync Settings").color(self.colors.text_highlight));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Sync Interval (minutes):");
                ui.add(egui::DragValue::new(&mut self.sync_interval)
                    .clamp_range(15..=1440));
            });

            ui.add_space(16.0);
            ui.heading(RichText::new("Article Settings").color(self.colors.text_highlight));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Article Retention (days):");
                ui.add(egui::DragValue::new(&mut self.article_retention_days)
                    .clamp_range(1..=365));
            });

            ui.add_space(16.0);

            if ui.button("Save Changes").clicked() {
                // Handle the Result directly instead of using the ? operator
                if let Err(e) = self.save_settings() {
                    eprintln!("Error saving settings: {}", e);
                    // Optionally show error to the user via UI
                } else {
                    self.show = false;
                }
            }
        });

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