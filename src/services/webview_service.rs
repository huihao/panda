use std::sync::Arc;
use anyhow::Result;
use log::error;
use url::Url;

use crate::models::article::Article;

/// Service for managing web view content
pub struct WebViewService {
    current_article: Option<Article>,
}

impl WebViewService {
    /// Creates a new web view service
    pub fn new() -> Self {
        Self {
            current_article: None,
        }
    }
    
    /// Loads an article into the web view
    pub fn load_article(&mut self, article: &Article) -> Result<()> {
        self.current_article = Some(article.clone());
        Ok(())
    }
    
    /// Gets the current article
    pub fn get_current_article(&self) -> Option<&Article> {
        self.current_article.as_ref()
    }
    
    /// Clears the current article
    pub fn clear(&mut self) {
        self.current_article = None;
    }
    
    /// Gets the HTML content for the current article
    pub fn get_html_content(&self) -> Result<String> {
        let article = self.current_article.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No article loaded"))?;
            
        let mut html = String::new();
        
        // Add header
        html.push_str(&format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        h1 {{
            font-size: 2em;
            margin-bottom: 0.5em;
        }}
        .meta {{
            color: #666;
            font-size: 0.9em;
            margin-bottom: 1em;
        }}
        .content {{
            font-size: 1.1em;
        }}
        img {{
            max-width: 100%;
            height: auto;
        }}
        a {{
            color: #0066cc;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="meta">
        <span>By {}</span>
        <span> • </span>
        <span>{}</span>
    </div>
    <div class="content">
"#,
            article.title,
            article.title,
            article.author.as_deref().unwrap_or("Unknown"),
            article.published_at.format("%Y-%m-%d %H:%M").to_string(),
        ));
        
        // Add content
        if let Some(content) = &article.content {
            html.push_str(content);
        } else {
            html.push_str("<p>No content available.</p>");
        }
        
        // Add footer
        html.push_str(r#"
    </div>
    <div class="meta">
        <a href="#" onclick="window.open('"#);
        html.push_str(&article.url.to_string());
        html.push_str(r#"', '_blank'); return false;">Read original article</a>
    </div>
</body>
</html>"#);
        
        Ok(html)
    }
} 