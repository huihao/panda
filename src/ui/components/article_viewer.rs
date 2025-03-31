use egui::{Color32, RichText, Ui, Context};
use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::base::repository::ArticleRepository;
use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::services::webview::WebViewService;
use crate::services::rss::RssService;
use crate::ui::styles::AppColors;

/// Component for viewing article content
pub struct ArticleViewer {
    article_repository: Arc<dyn ArticleRepository>,
    webview_service: Arc<WebViewService>,
    current_article: Option<Article>,
    colors: AppColors,
    rss_service: Arc<RssService>,
}

impl ArticleViewer {
    /// Creates a new article viewer
    pub fn new(
        article_repository: Arc<dyn ArticleRepository>,
        webview_service: Arc<WebViewService>,
        rss_service: Arc<RssService>,
        colors: AppColors,
    ) -> Self {
        Self {
            article_repository,
            webview_service,
            current_article: None,
            colors,
            rss_service,
        }
    }

    /// Shows the article viewer with just the article parameter
    pub fn show(&mut self, article: &Article) -> Result<()> {
        // Set the current article
        self.current_article = Some(article.clone());
        
        // Mark the article as read if needed
        if article.read_status == ReadStatus::Unread {
            if let Err(e) = self.mark_as_read(article.id.clone()) {
                log::error!("Failed to mark article as read: {}", e);
            }
        }
        
        // Initialize the webview service with the article content
        if let Some(content) = &article.content {
            self.webview_service.load_content(content)?;
        } else if let Some(summary) = &article.summary {
            self.webview_service.load_content(summary)?;
        } else {
            self.webview_service.load_content("No content available.")?;
        }
        
        Ok(())
    }

    /// The original show method that displays in the UI context
    pub fn show_in_ui(&mut self, ctx: &Context, article: &Article) -> Result<()> {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.heading(RichText::new(&article.title).color(self.colors.text));

            // Metadata
            ui.horizontal(|ui| {
                if let Some(author) = &article.author {
                    ui.label(RichText::new(format!("By {}", author)).color(self.colors.text_secondary));
                }
                if let Some(published_at) = article.published_at {
                    ui.label(RichText::new(published_at.format("%Y-%m-%d %H:%M").to_string())
                        .color(self.colors.text_secondary));
                }
            });

            // Content
            ui.separator();
            if let Some(content) = &article.content {
                ui.label(RichText::new(content).color(self.colors.text));
            } else if let Some(summary) = &article.summary {
                ui.label(RichText::new(summary).color(self.colors.text));
            }

            // Actions
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Open in Browser").color(self.colors.accent)).clicked() {
                    if let Err(e) = open::that(&article.url) {
                        log::error!("Failed to open URL: {}", e);
                    }
                }
                if ui.button(RichText::new("Mark as Read").color(self.colors.accent)).clicked() {
                    if let Err(e) = self.mark_as_read(article.id.clone()) {
                        log::error!("Failed to mark article as read: {}", e);
                    }
                }
                if ui.button(RichText::new("Add to Favorites").color(self.colors.accent)).clicked() {
                    if let Err(e) = self.add_to_favorites(article.id.clone()) {
                        log::error!("Failed to add article to favorites: {}", e);
                    }
                }
            });
        });

        Ok(())
    }

    /// Loads an article by its ID
    pub fn load_article(&mut self, article: &Article) -> Result<()> {
        let mut article = article.clone();
        if article.read_status == ReadStatus::Unread {
            article.read_status = ReadStatus::Read;
            if let Err(e) = self.article_repository.update_article(&article) {
                eprintln!("Failed to mark article as read: {}", e);
            }
        }
        self.current_article = Some(article);
        Ok(())
    }

    /// Clears the current article
    pub fn clear(&mut self) {
        self.current_article = None;
    }

    /// Gets the currently displayed article
    pub fn get_current_article(&self) -> Option<&Article> {
        self.current_article.as_ref()
    }

    /// Sets the current article
    pub fn set_article(&mut self, article: Article) {
        self.current_article = Some(article);
    }

    /// Marks an article as read
    fn mark_as_read(&self, article_id: ArticleId) -> Result<()> {
        self.article_repository.mark_as_read(&article_id)?;
        Ok(())
    }

    /// Adds an article to favorites
    fn add_to_favorites(&self, article_id: ArticleId) -> Result<()> {
        self.article_repository.add_to_favorites(&article_id)?;
        Ok(())
    }
}

fn format_date(date: DateTime<Utc>) -> String {
    date.format("%B %d, %Y %H:%M").to_string()
}