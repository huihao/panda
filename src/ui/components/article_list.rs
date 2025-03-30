use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use egui::{Ui, ScrollArea, RichText, Label, Vec2, Color32};
use log::error;

use crate::base::repository::ArticleRepository;
use crate::models::article::{Article, ArticleId, ReadStatus, CategoryId, FeedId, Tag};
use crate::ui::styles::{AppColors, DEFAULT_PADDING, DEFAULT_SPACING};
use crate::core::repository::ArticleRepository as CoreArticleRepository;
use crate::services::RssService;

/// Article sort order
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArticleSortOrder {
    DateDesc,
    DateAsc,
    TitleAsc,
    TitleDesc,
}

/// Component for displaying a list of articles
pub struct ArticleList {
    article_repository: Arc<dyn ArticleRepository>,
    rss_service: Arc<RssService>,
    colors: AppColors,
    selected_article: Option<ArticleId>,
    articles: Vec<Article>,
    sort_order: ArticleSortOrder,
    scroll_to_index: Option<usize>,
    sort_by_date: bool,
}

impl ArticleList {
    /// Creates a new article list
    pub fn new(
        article_repository: Arc<dyn ArticleRepository>,
        rss_service: Arc<RssService>,
        colors: AppColors,
    ) -> Self {
        Self {
            article_repository,
            rss_service,
            colors,
            selected_article: None,
            articles: Vec::new(),
            sort_order: ArticleSortOrder::DateDesc,
            scroll_to_index: None,
            sort_by_date: true,
        }
    }
    
    /// Renders the article list UI
    pub fn show(&mut self, ui: &mut Ui) -> Result<Option<ArticleId>> {
        let mut selected = None;

        ui.vertical(|ui| {
            // Sort buttons
            ui.horizontal(|ui| {
                ui.label(RichText::new("Sort by:").color(self.colors.text));
                ui.radio_value(&mut self.sort_order, ArticleSortOrder::DateDesc, "Newest");
                ui.radio_value(&mut self.sort_order, ArticleSortOrder::DateAsc, "Oldest");
                ui.radio_value(&mut self.sort_order, ArticleSortOrder::TitleAsc, "Title (A-Z)");
                ui.radio_value(&mut self.sort_order, ArticleSortOrder::TitleDesc, "Title (Z-A)");
            });

            // Article list
            for article in &self.articles {
                let is_selected = self.selected_article.as_ref() == Some(&article.id);
                let text_color = if is_selected {
                    self.colors.accent
                } else if article.read_status == "Unread" {
                    self.colors.text
                } else {
                    self.colors.text_secondary
                };

                if ui.button(RichText::new(&article.title).color(text_color)).clicked() {
                    self.selected_article = Some(article.id.clone());
                    selected = Some(article.id.clone());
                }
            }
        });

        Ok(selected)
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
        self.articles = self.article_repository.get_all_articles()?;
        if self.sort_by_date {
            self.articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        } else {
            self.articles.sort_by(|a, b| a.title.cmp(&b.title));
        }
        Ok(())
    }

    pub fn get_selected_article(&self) -> Option<&Article> {
        self.selected_article.as_ref().and_then(|id| {
            self.articles.iter().find(|a| a.id == *id)
        })
    }

    pub fn clear_selection(&mut self) {
        self.selected_article = None;
    }

    pub fn set_sort_by_date(&mut self, sort_by_date: bool) -> Result<()> {
        self.sort_by_date = sort_by_date;
        self.refresh()
    }

    pub fn filter_by_tag(&mut self, tag: &Tag) -> Result<()> {
        self.articles = self.article_repository.get_articles_by_tag(&tag.id)?;
        if self.sort_by_date {
            self.articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        } else {
            self.articles.sort_by(|a, b| a.title.cmp(&b.title));
        }
        Ok(())
    }

    pub fn set_articles(&mut self, articles: Vec<Article>) {
        self.articles = articles;
        self.sort_articles();
    }

    pub fn sort_articles(&mut self) {
        match self.sort_order {
            ArticleSortOrder::DateDesc => {
                self.articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
            }
            ArticleSortOrder::DateAsc => {
                self.articles.sort_by(|a, b| a.published_at.cmp(&b.published_at));
            }
            ArticleSortOrder::TitleAsc => {
                self.articles.sort_by(|a, b| a.title.cmp(&b.title));
            }
            ArticleSortOrder::TitleDesc => {
                self.articles.sort_by(|a, b| b.title.cmp(&a.title));
            }
        }
    }

    pub fn clear(&mut self) {
        self.articles.clear();
        self.selected_article = None;
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