use anyhow::Result;
use std::sync::Arc;
use crate::models::article::{Article, ArticleId};
use crate::data::ArticleRepository;

#[derive(Clone)]
pub struct ArticleService {
    repository: Arc<dyn ArticleRepository>,
}

impl ArticleService {
    pub fn new(repository: Arc<dyn ArticleRepository>) -> Self {
        Self { repository }
    }

    pub fn get_article(&self, id: &str) -> Result<Option<Article>> {
        self.repository.get_article(&ArticleId(id.to_string()))
    }
}
