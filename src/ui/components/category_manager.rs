use std::sync::Arc;
use anyhow::Result;
use egui::{Ui, Window, TextEdit, Button, ScrollArea, CollapsingHeader};
use log::error;

use crate::core::CategoryRepository;
use crate::models::{Category, CategoryId};
use crate::ui::styles::{AppColors, DEFAULT_PADDING};

/// Category management dialog component
pub struct CategoryManager {
    category_repository: Arc<dyn CategoryRepository>,
    colors: AppColors,
    is_open: bool,
    categories: Vec<Category>,
    new_category_name: String,
    error_message: Option<String>,
    editing_category: Option<(CategoryId, String)>,
    selected_category: Option<CategoryId>,
    selected_parent: Option<CategoryId>,
    is_processing: bool,
}

impl CategoryManager {
    /// Creates a new category manager
    pub fn new(category_repository: Arc<dyn CategoryRepository>) -> Self {
        Self {
            category_repository,
            colors: AppColors::default(),
            is_open: false,
            categories: Vec::new(),
            new_category_name: String::new(),
            error_message: None,
            editing_category: None,
            selected_category: None,
            selected_parent: None,
            is_processing: false,
        }
    }
    
    /// Shows the category manager dialog
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        if !self.is_open {
            return Ok(());
        }
        
        // Refresh categories list
        self.categories = self.category_repository.get_all_categories()?;
        
        Window::new("Manage Categories")
            .collapsible(false)
            .resizable(true)
            .min_width(400.0)
            .min_height(300.0)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    // Add new category section
                    ui.group(|ui| {
                        ui.label("Add New Category");
                        ui.horizontal(|ui| {
                            let response = ui.add(
                                TextEdit::singleline(&mut self.new_category_name)
                                    .hint_text("Enter category name")
                            );
                            
                            if ui.add_enabled(
                                !self.new_category_name.is_empty() && !self.is_processing,
                                Button::new("Add")
                            ).clicked() {
                                self.add_category();
                            }
                            
                            if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                                self.add_category();
                            }
                        });
                        
                        // Parent category selection
                        ui.horizontal(|ui| {
                            ui.label("Parent Category:");
                            egui::ComboBox::from_label("")
                                .selected_text(
                                    self.get_category_name(self.selected_parent.as_ref())
                                        .unwrap_or("None")
                                )
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(
                                        &mut self.selected_parent,
                                        None,
                                        "None"
                                    ).clicked() {}
                                    
                                    for category in &self.categories {
                                        if ui.selectable_value(
                                            &mut self.selected_parent,
                                            Some(category.id.clone()),
                                            &category.name
                                        ).clicked() {}
                                    }
                                });
                        });
                    });
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Error message
                    if let Some(error) = &self.error_message {
                        ui.colored_label(self.colors.error, error);
                        ui.add_space(DEFAULT_PADDING);
                    }
                    
                    // Categories list
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.show_category_tree(ui, None, 0)?;
                        });
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Close button
                    if ui.button("Close").clicked() {
                        self.close();
                    }
                });
            });
        
        Ok(())
    }
    
    /// Opens the category manager dialog
    pub fn open(&mut self) {
        self.is_open = true;
        self.refresh();
    }
    
                                        });
                                    }
                                });
                            }
                        });
                    
                    ui.add_space(DEFAULT_PADDING);
                    
                    // Close button
                    if ui.button("Close").clicked() {
                        self.close();
                    }
                });
            });
        
        Ok(())
    }
    
    /// Opens the category manager dialog
    pub fn open(&mut self) {
        self.is_open = true;
        self.new_category_name.clear();
        self.error_message = None;
        self.editing_category = None;
    }
    
    /// Closes the category manager dialog
    pub fn close(&mut self) {
        self.is_open = false;
        self.new_category_name.clear();
        self.error_message = None;
        self.editing_category = None;
    }
    
    /// Returns whether the dialog is currently open
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    
    /// Adds a new category
    fn add_category(&mut self) -> Result<()> {
        let name = self.new_category_name.trim();
        if name.is_empty() {
            return Err(anyhow::anyhow!("Category name is required"));
        }
        
        if self.categories.iter().any(|c| c.name == name) {
            return Err(anyhow::anyhow!("A category with this name already exists"));
        }
        
        let category = Category {
            id: CategoryId::new(),
            name: name.to_string(),
        };
        
        self.category_repository.create_category(&category)?;
        self.categories.push(category);
        
        Ok(())
    }
    
    /// Saves edits to an existing category
    fn save_category_edit(&mut self, category: &Category) -> Result<()> {
        if let Some((editing_id, ref name)) = &self.editing_category {
            let name = name.trim();
            if name.is_empty() {
                return Err(anyhow::anyhow!("Category name is required"));
            }
            
            if self.categories.iter().any(|c| c.name == name && c.id != *editing_id) {
                return Err(anyhow::anyhow!("A category with this name already exists"));
            }
            
            let mut updated_category = category.clone();
            updated_category.name = name.to_string();
            
            self.category_repository.update_category(&updated_category)?;
            
            if let Some(idx) = self.categories.iter().position(|c| c.id == *editing_id) {
                self.categories[idx] = updated_category;
            }
        }
        
        Ok(())
    }
    
    /// Deletes a category
    fn delete_category(&mut self, category: &Category) -> Result<()> {
        // Remove category from all feeds in this category
        let mut feeds = self.feed_repository.get_feeds_by_category(&category.id)?;
        for mut feed in feeds {
            feed.category_id = None;
            feed.category_name = None;
            self.feed_repository.update_feed(&feed)?;
        }
        
        // Delete the category
        self.category_repository.delete_category(&category.id)?;
        
        if let Some(idx) = self.categories.iter().position(|c| c.id == category.id) {
            self.categories.remove(idx);
        }
        
        Ok(())
    }
}