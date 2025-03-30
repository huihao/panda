use egui::{Color32, RichText, Ui};
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

    /// Renders the article viewer UI
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        if let Some(article) = &self.current_article {
            // Article header
            ui.vertical(|ui| {
                ui.heading(RichText::new(&article.title)
                    .color(self.colors.text)
                    .size(24.0));

                if let Some(author) = &article.author {
                    ui.label(RichText::new(format!("By {}", author))
                        .color(self.colors.text_secondary));
                }

                if let Some(published_at) = article.published_at {
                    ui.label(RichText::new(published_at.format("%Y-%m-%d %H:%M").to_string())
                        .color(self.colors.text_secondary));
                }

                ui.add_space(16.0);

                if let Some(summary) = &article.summary {
                    ui.label(RichText::new(summary)
                        .color(self.colors.text));
                }

                ui.add_space(16.0);

                if let Some(content) = &article.content {
                    ui.label(RichText::new(content)
                        .color(self.colors.text));
                }
            });

            // Article content
            ui.vertical(|ui| {
                let html = self.webview_service.render_article(article);
                ui.label(RichText::new(html).text_style(egui::TextStyle::Body));
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("No article selected").color(self.colors.text_secondary));
            });
        }

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
}

fn format_date(date: DateTime<Utc>) -> String {
    date.format("%B %d, %Y %H:%M").to_string()
}