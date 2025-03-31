use egui::{Ui, RichText};
use std::sync::Arc;
use anyhow::Result;
use log::error;

use crate::models::article::{Article, ArticleId, ReadStatus};
use crate::base::repository::ArticleRepository;
use crate::services::rss::RssService;
use crate::ui::styles::AppColors;

pub enum ArticleSortOrder {
    NewestFirst,
    OldestFirst,
    Unread,
}

pub struct ArticleList {
    article_repository: Arc<dyn ArticleRepository>,
    rss_service: Arc<RssService>,
    colors: AppColors,
    articles: Vec<Article>,
    sort_order: ArticleSortOrder,
    selected_article: Option<ArticleId>,
}

impl ArticleList {
    pub fn new(
        article_repository: Arc<dyn ArticleRepository>,
        rss_service: Arc<RssService>,
        colors: AppColors,
    ) -> Self {
        Self {
            article_repository,
            rss_service,
            colors,
            articles: Vec::new(),
            sort_order: ArticleSortOrder::NewestFirst,
            selected_article: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Result<Option<ArticleId>> {
        let mut selected = None;

        for article in &self.articles {
            let text = format!(
                "{}\n{}",
                article.title,
                article.summary.as_deref().unwrap_or("")
            );

            let mut color = self.colors.text;
            if article.read_status == ReadStatus::Unread {
                color = self.colors.text_highlight;
            }

            if ui.add(
                egui::Button::new(RichText::new(&text).color(color))
                    .wrap(true)
                    .min_size(egui::vec2(0.0, 60.0))
            ).clicked() {
                selected = Some(article.id.clone());
                self.selected_article = Some(article.id.clone());
            }
        }

        Ok(selected)
    }

    pub async fn load_articles(&mut self, feed_id: Option<String>) -> Result<()> {
        self.articles = match feed_id {
            Some(id) => self.rss_service.get_articles_by_feed(&id.into()).await?,
            None => self.rss_service.get_all_articles().await?,
        };

        self.sort_articles();
        Ok(())
    }

    pub fn set_sort_order(&mut self, order: ArticleSortOrder) {
        self.sort_order = order;
        self.sort_articles();
    }

    fn sort_articles(&mut self) {
        match self.sort_order {
            ArticleSortOrder::NewestFirst => {
                self.articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
            }
            ArticleSortOrder::OldestFirst => {
                self.articles.sort_by(|a, b| a.published_at.cmp(&b.published_at));
            }
            ArticleSortOrder::Unread => {
                self.articles.sort_by(|a, b| {
                    (b.read_status == ReadStatus::Unread)
                        .cmp(&(a.read_status == ReadStatus::Unread))
                        .then_with(|| b.published_at.cmp(&a.published_at))
                });
            }
        }
    }

    pub fn get_selected_article(&self) -> Option<ArticleId> {
        self.selected_article.clone()
    }

    pub fn clear_selection(&mut self) {
        self.selected_article = None;
    }
}