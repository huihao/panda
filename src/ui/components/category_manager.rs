use std::sync::Arc;
use egui::{Ui, Window, TextEdit, Button, RichText, Color32};
use anyhow::Result;
use log::error;

use crate::base::repository::CategoryRepository;
use crate::models::category::Category;
use crate::ui::styles::AppColors;

/// Component for managing categories
pub struct CategoryManager {
    category_repository: Arc<dyn CategoryRepository>,
    colors: AppColors,
    visible: bool,
    name_input: String,
    description_input: String,
    error_message: Option<String>,
}

impl CategoryManager {
    /// Creates a new category manager
    pub fn new(category_repository: Arc<dyn CategoryRepository>, colors: AppColors) -> Self {
        Self {
            category_repository,
            colors,
            visible: false,
            name_input: String::new(),
            description_input: String::new(),
            error_message: None,
        }
    }
    
    /// Shows the category manager dialog
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        if self.visible {
            Window::new("Add Category")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Category Name").color(self.colors.text));
                        ui.add(TextEdit::singleline(&mut self.name_input)
                            .hint_text("Enter category name")
                            .desired_width(300.0));
                        
                        ui.label(RichText::new("Description").color(self.colors.text));
                        ui.add(TextEdit::multiline(&mut self.description_input)
                            .hint_text("Enter category description")
                            .desired_width(300.0));
                        
                        if let Some(error) = &self.error_message {
                            ui.label(RichText::new(error).color(self.colors.error));
                        }
                        
                        ui.horizontal(|ui| {
                            if ui.button("Add").clicked() {
                                if let Err(e) = self.add_category() {
                                    error!("Failed to add category: {}", e);
                                    self.error_message = Some(e.to_string());
                                } else {
                                    self.visible = false;
                                    self.name_input.clear();
                                    self.description_input.clear();
                                    self.error_message = None;
                                }
                            }
                            
                            if ui.button("Cancel").clicked() {
                                self.visible = false;
                                self.name_input.clear();
                                self.description_input.clear();
                                self.error_message = None;
                            }
                        });
                    });
                });
        }
        Ok(())
    }
    
    /// Adds a new category
    fn add_category(&self) -> Result<()> {
        if self.name_input.is_empty() {
            return Err(anyhow::anyhow!("Name cannot be empty"));
        }
        
        let category = Category::new(
            self.name_input.clone(),
            if self.description_input.is_empty() { None } else { Some(self.description_input.clone()) }
        );
        
        self.category_repository.save_category(&category)?;
        Ok(())
    }
}