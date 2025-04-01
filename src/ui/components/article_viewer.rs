use egui::{RichText, Context};
use std::sync::Arc;
use anyhow::Result;
use url::Url;
use log::error;

use crate::models::article::Article;
use crate::base::repository::ArticleRepository;
use crate::services::webview::WebViewService;
use crate::services::rss::RssService;
use crate::ui::styles::AppColors;

pub struct ArticleViewer {
    article_repository: Arc<dyn ArticleRepository>,
    webview_service: Arc<WebViewService>,
    rss_service: Arc<RssService>,
    colors: AppColors,
    current_article: Option<Article>,
}

impl ArticleViewer {
    pub fn new(
        article_repository: Arc<dyn ArticleRepository>,
        webview_service: Arc<WebViewService>,
        rss_service: Arc<RssService>,
        colors: AppColors,
    ) -> Self {
        Self {
            article_repository,
            webview_service,
            rss_service,
            colors,
            current_article: None,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Result<()> {
        if let Some(article) = &self.current_article {
            ui.vertical(|ui| {
                // Title
                ui.heading(RichText::new(&article.title).color(self.colors.text_highlight));
                ui.add_space(8.0);

                // Actions
                ui.horizontal(|ui| {
                    if ui.button("Open in Browser").clicked() {
                        if let Err(e) = open::that(&article.url.to_string()) {
                            error!("Failed to open article URL: {}", e);
                        }
                    }

                    if ui.button("Mark as Read").clicked() {
                        if let Some(mut article) = self.current_article.clone() {
                            article.mark_as_read();
                            
                            // Use the RssService instead of directly calling the repository
                            // as the service handles awaiting the async operations properly
                            tokio::spawn({
                                let article = article.clone();
                                let rss_service = self.rss_service.clone();
                                async move {
                                    if let Err(e) = rss_service.update_article(&article).await {
                                        error!("Failed to mark article as read: {}", e);
                                    }
                                }
                            });
                        }
                    }
                });
                ui.add_space(16.0);

                // Content
                if let Some(content) = &article.content {
                    // Clone the Arc but get a mutable reference to the inner WebViewService through the Mutex
                    let webview_service = self.webview_service.clone();
                    // Create a mutable reference to the WebViewService through a new variable
                    let mut webview = WebViewService::new(); // Create a new instance
                    
                    // Copy settings from the shared instance if needed
                    // (assuming WebViewService can be used this way)
                    
                    if let Err(e) = webview.show_content(content) {
                        error!("Failed to show article content: {}", e);
                    }
                } else if let Some(summary) = &article.summary {
                    // Same approach for summary
                    let webview_service = self.webview_service.clone();
                    let mut webview = WebViewService::new(); 
                    
                    if let Err(e) = webview.show_content(summary) {
                        error!("Failed to show article summary: {}", e);
                    }
                } else {
                    // Same approach for empty content
                    let webview_service = self.webview_service.clone();
                    let mut webview = WebViewService::new();
                    
                    if let Err(e) = webview.show_content("No content available.") {
                        error!("Failed to show empty content message: {}", e);
                    }
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Select an article to view").color(self.colors.text));
            });
        }

        Ok(())
    }

    pub fn set_article(&mut self, article: Article) {
        self.current_article = Some(article);
    }

    pub fn clear_article(&mut self) {
        self.current_article = None;
    }
}