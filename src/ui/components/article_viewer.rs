use std::sync::Arc;
use egui::{Ui, TopBottomPanel, ScrollArea, Button, RichText, Widget};
use anyhow::Result;
use chrono::Utc;
use log::error;

use crate::core::ArticleRepository;
use crate::models::{Article, ArticleId, Tag};
use crate::services::WebViewService;
use crate::ui::styles::{AppColors, DEFAULT_PADDING};

/// Component for viewing article content
pub struct ArticleViewer {
    article_repository: Arc<dyn ArticleRepository>,
    webview_service: Arc<WebViewService>,
    current_article: Option<Article>,
    new_tag_name: String,
    colors: AppColors,
}

impl ArticleViewer {
    /// Creates a new article viewer
    pub fn new(
        article_repository: Arc<dyn ArticleRepository>,
        webview_service: Arc<WebViewService>,
    ) -> Self {
        Self {
            article_repository,
            webview_service,
            current_article: None,
            new_tag_name: String::new(),
            colors: AppColors::default(),
        }
    }

    /// Renders the article viewer UI
    pub fn ui(&mut self, ui: &mut Ui) -> Result<()> {
        if let Some(article) = &mut self.current_article {
            // Article toolbar
            TopBottomPanel::top("article_toolbar").show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    // Navigation controls
                    if ui.button("⬅ Back").clicked() {
                        if let Err(e) = self.webview_service.navigate_back() {
                            error!("Failed to navigate back: {}", e);
                        }
                    }
                    
                    if ui.button("➡ Forward").clicked() {
                        if let Err(e) = self.webview_service.navigate_forward() {
                            error!("Failed to navigate forward: {}", e);
                        }
                    }
                    
                    if ui.button("🔄 Refresh").clicked() {
                        if let Err(e) = self.webview_service.refresh() {
                            error!("Failed to refresh page: {}", e);
                        }
                    }
                    
                    ui.separator();
                    
                    // Original article link
                    if ui.button("🌐 Open in Browser").clicked() {
                        if let Err(e) = self.webview_service.navigate_to_url(&article.url) {
                            error!("Failed to open URL: {}", e);
                        }
                    }
                    
                    // Favorite toggle
                    let favorite_text = if article.is_favorite { "★ Favorited" } else { "☆ Favorite" };
                    let favorite_button = Button::new(RichText::new(favorite_text)
                        .color(if article.is_favorite {
                            self.colors.warning
                        } else {
                            self.colors.text
                        }));
                    
                    if favorite_button.ui(ui).clicked() {
                        article.is_favorite = !article.is_favorite;
                        if let Err(e) = self.article_repository.update_article(article) {
                            error!("Failed to update article favorite status: {}", e);
                        }
                    }
                    
                    ui.separator();
                    
                    // Tags
                    ui.label("Tags:");
                    let mut tag_to_remove = None;
                    
                    for tag in &article.tags {
                        let tag_color = tag.color
                            .as_ref()
                            .and_then(|c| self.parse_color(c))
                            .unwrap_or(self.colors.secondary);
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&tag.name).color(tag_color));
                            if ui.small_button("×").clicked() {
                                tag_to_remove = Some(tag.clone());
                            }
                        });
                    }
                    
                    // Handle tag removal
                    if let Some(tag) = tag_to_remove {
                        article.tags.remove(&tag);
                        if let Err(e) = self.article_repository.update_article(article) {
                            error!("Failed to remove tag: {}", e);
                        }
                    }
                    
                    // Add new tag
                    ui.horizontal(|ui| {
                        let text_edit = ui.text_edit_singleline(&mut self.new_tag_name);
                        if (text_edit.lost_focus() && ui.input().key_pressed(egui::Key::Enter)) || 
                           ui.button("+").clicked() {
                            if !self.new_tag_name.is_empty() {
                                let tag = Tag {
                                    id: format!("tag_{}", rand::random::<u64>()),
                                    name: self.new_tag_name.clone(),
                                    color: None,
                                };
                                article.tags.insert(tag);
                                if let Err(e) = self.article_repository.update_article(article) {
                                    error!("Failed to add tag: {}", e);
                                }
                                self.new_tag_name.clear();
                            }
                        }
                    });
                });
            });
            
            // Article content
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Initialize webview
                    let html_content = match (article.content.as_ref(), article.summary.as_ref()) {
                        (Some(content), _) => content,
                        (None, Some(summary)) => summary,
                        (None, None) => "No content available",
                    };
                    
                    let sanitized_content = self.webview_service.sanitize_content(html_content)?;
                    let formatted_date = article.published_date
                        .map(|dt| dt.format("%B %d, %Y %H:%M").to_string());
                    
                    let article_html = self.webview_service.create_article_html(
                        &article.title,
                        &sanitized_content,
                        formatted_date.as_deref(),
                        article.author.as_deref(),
                    );
                    
                    self.webview_service.render_html(&article_html)?;
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("Select an article to read");
            });
        }
        
        Ok(())
    }
    
    /// Loads an article by ID
    pub fn load_article(&mut self, article_id: &ArticleId) -> Result<()> {
        if let Some(mut article) = self.article_repository.get_article_by_id(article_id)? {
            // Mark as read if not already
            if !matches!(article.read_status, ReadStatus::Read) {
                article.read_status = ReadStatus::Read;
                self.article_repository.update_article(&article)?;
            }
            self.current_article = Some(article);
        }
        Ok(())
    }
    
    /// Gets the currently displayed article
    pub fn get_article(&self) -> Option<&Article> {
        self.current_article.as_ref()
    }
    
    /// Clears the current article
    pub fn clear(&mut self) {
        self.current_article = None;
    }
    
    /// Parses a color string (hex format) into a Color32
    fn parse_color(&self, color: &str) -> Option<egui::Color32> {
        if !color.starts_with('#') || color.len() != 7 {
            return None;
        }
        
        let r = u8::from_str_radix(&color[1..3], 16).ok()?;
        let g = u8::from_str_radix(&color[3..5], 16).ok()?;
        let b = u8::from_str_radix(&color[5..7], 16).ok()?;
        
        Some(egui::Color32::from_rgb(r, g, b))
    }
}