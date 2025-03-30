use std::sync::Arc;
use crate::services::rss::RssService;
use crate::ui::theme::AppColors;

pub struct Context {
    pub rss_service: Arc<RssService>,
    pub colors: AppColors,
}

impl Context {
    pub fn new(rss_service: Arc<RssService>) -> Self {
        Self {
            rss_service,
            colors: AppColors::default(),
        }
    }
} 