use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use egui::{Ui, ScrollArea, RichText, Label, Vec2};

use crate::core::ArticleRepository;
use crate::models::{Article, ArticleId, ReadStatus};
use crate::ui::styles::{AppColors, DEFAULT_PADDING, DEFAULT_SPACING};

/// Article sort order
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArticleSortOrder {
    NewestFirst,
    OldestFirst,
}

/// Component for displaying a list of articles
pub struct ArticleList {
    article_repository: Arc<dyn ArticleRepository>,
    colors: AppColors,
    selected_article: Option<ArticleId>,
    pub sort_order: ArticleSortOrder,
    scroll_to_index: Option<usize>,
}

impl ArticleList {
    /// Creates a new article list
    pub fn new(article_repository: Arc<dyn ArticleRepository>) -> Self {
        Self {
            article_repository,
            colors: AppColors::default(),
            selected_article: None,
            sort_order: ArticleSortOrder::NewestFirst,
            scroll_to_index: None,
        }
    }
    
    /// Renders the article list UI
    pub fn ui(&mut self, ui: &mut Ui, articles: &[Article]) -> Result<Option<ArticleId>> {
        let mut selected = None;
        
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Sort articles by date
                let mut sorted_articles = articles.to_vec();
                sorted_articles.sort_by(|a, b| {
                    let time_a = a.published_date.unwrap_or_else(|| a.saved_date);
                    let time_b = b.published_date.unwrap_or_else(|| b.saved_date);
                    match self.sort_order {
                        ArticleSortOrder::NewestFirst => time_b.cmp(&time_a),
                        ArticleSortOrder::OldestFirst => time_a.cmp(&time_b),
                    }
                });
                
                // Display articles
                for (index, article) in sorted_articles.iter().enumerate() {
                    let is_selected = self.selected_article.as_ref() == Some(&article.id);
                    let is_read = matches!(article.read_status, ReadStatus::Read);
                    
                    let height = if is_selected { 100.0 } else { 80.0 };
                    
                    // Article card
                    ui.add_space(DEFAULT_SPACING);
                    egui::Frame::none()
                        .fill(if is_selected {
                            self.colors.foreground
                        } else {
                            self.colors.background
                        })
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.allocate_space(Vec2::new(ui.available_width(), height));
                            
                            // Article header
                            ui.horizontal(|ui| {
                                // Favorite indicator
                                if article.is_favorite {
                                    ui.add(Label::new(
                                        RichText::new("★")
                                            .color(self.colors.warning)
                                            .size(16.0)
                                    ));
                                }
                                
                                // Article title
                                let title_text = RichText::new(&article.title)
                                    .size(14.0)
                                    .color(if is_read {
                                        self.colors.text_dimmed
                                    } else {
                                        self.colors.text
                                    });
                                
                                ui.add(Label::new(title_text));
                            });
                            
                            // Article metadata
                            ui.horizontal(|ui| {
                                // Published date
                                if let Some(date) = article.published_date {
                                    ui.add(Label::new(
                                        RichText::new(format_date(date))
                                            .size(12.0)
                                            .color(self.colors.text_dimmed)
                                    ));
                                }
                                
                                ui.add(Label::new(" • "));
                                
                                // Author
                                if let Some(author) = &article.author {
                                    ui.add(Label::new(
                                        RichText::new(author)
                                            .size(12.0)
                                            .color(self.colors.text_dimmed)
                                    ));
                                }
                            });
                            
                            // Article summary (only for selected item)
                            if is_selected {
                                if let Some(summary) = &article.summary {
                                    ui.add(Label::new(
                                        RichText::new(truncate_text(summary, 150))
                                            .size(12.0)
                                            .color(self.colors.text_dimmed)
                                    ));
                                }
                            }
                            
                            // Make entire area clickable
                            if ui.interact(ui.min_rect(), ui.id(), egui::Sense::click()).clicked() {
                                self.selected_article = Some(article.id.clone());
                                selected = Some(article.id.clone());
                            }
                        });
                }
            });
        
        Ok(selected)
    }
    
    /// Sets the selected article
    pub fn set_selected_article(&mut self, article_id: Option<ArticleId>) {
        self.selected_article = article_id;
    }
    
    /// Gets all articles
    pub fn get_all_articles(&self) -> Result<Vec<Article>> {
        self.article_repository.get_all_articles()
    }
    
    /// Gets articles for a specific feed
    pub fn get_feed_articles(&self, feed_id: &FeedId) -> Result<Vec<Article>> {
        self.article_repository.get_articles_by_feed(feed_id)
    }
    
    /// Gets favorite articles
    pub fn get_favorite_articles(&self) -> Result<Vec<Article>> {
        self.article_repository.get_favorite_articles()
    }
    
    /// Gets articles for a specific tag
    pub fn get_articles_by_tag(&self, tag: &Tag) -> Result<Vec<Article>> {
        self.article_repository.get_articles_by_tag(tag)
    }
    
    /// Refreshes the article list
    pub fn refresh(&mut self) -> Result<()> {
        // Clear selection
        self.selected_article = None;
        Ok(())
    }
}

/// Formats a datetime for display
fn format_date(date: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(date);
    
    if diff.num_days() == 0 {
        if diff.num_hours() == 0 {
            if diff.num_minutes() == 0 {
                "just now".to_string()
            } else {
                format!("{}m ago", diff.num_minutes())
            }
        } else {
            format!("{}h ago", diff.num_hours())
        }
    } else if diff.num_days() < 7 {
        format!("{}d ago", diff.num_days())
    } else {
        date.format("%Y-%m-%d").to_string()
    }
}

/// Truncates text to the specified length
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}