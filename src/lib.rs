pub mod base;
pub mod data;
pub mod models;
pub mod services;
pub mod ui;
pub mod utils;

// Re-export repository traits
pub use base::repository::{
    ArticleRepository,
    CategoryRepository,
    FeedRepository,
    TagRepository,
};

// Re-export models
pub use models::{
    article::{Article, ArticleId},
    category::{Category, CategoryId},
    feed::{Feed, FeedId, FeedStatus},
    tag::{Tag, TagId},
};

// Re-export services selectively
pub use services::{
    article::ArticleService,
    opml::OpmlService,
    rss::RssService,
    webview::WebViewService,
};
