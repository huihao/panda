use anyhow::Result;
use wry::WebViewBuilder;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use std::sync::Arc;

use crate::models::article::Article;
use crate::services::article::ArticleService;

/// Service for handling web views to display article content
pub struct WebViewService {
    article_service: ArticleService,
}

impl WebViewService {
    /// Creates a new WebViewService instance
    pub fn new(article_service: ArticleService) -> Self {
        Self { article_service }
    }

    /// Creates a new webview to display the specified article
    pub fn create_article_view(&self, article_id: &str) -> Result<()> {
        let article = self.article_service.get_article(article_id)?
            .ok_or_else(|| anyhow::anyhow!("Article not found"))?;

        // Create an event loop - no ? operator as it returns EventLoop directly, not Result
        let event_loop = EventLoop::new();
        
        // Create a window
        let window = WindowBuilder::new()
            .with_title(&article.title)
            .with_resizable(true)
            .with_inner_size(LogicalSize::new(800.0, 600.0))
            .build(&event_loop)?;

        // Create a webview using the correct API pattern
        // First create builder
        let builder = WebViewBuilder::new()
            .with_url("about:blank")  // No ? here as this returns WebViewBuilder, not Result
            .with_html(self.render_article(&article));  // No ? here as this returns WebViewBuilder, not Result
            
        // Then build the webview and apply ? to the Result
        let _webview = builder.build(&window)?;  // Apply ? only to the build method which returns Result

        // Run the event loop with the correct API
        event_loop.run(move |event, event_loop, control_flow| {
            *control_flow = ControlFlow::Wait;
            
            match event {
                Event::WindowEvent { 
                    event: WindowEvent::CloseRequested, 
                    .. 
                } => {
                    *control_flow = ControlFlow::Exit;
                },
                _ => (),
            }
        });

        // Note: code below this point will never be reached because event_loop.run() takes ownership
        // and never returns. This is expected behavior for GUI applications.
        #[allow(unreachable_code)]
        Ok(())
    }

    pub fn render_article(&self, article: &Article) -> String {
        let mut html = String::new();
        
        // Add HTML header
        html.push_str(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>"#);
        html.push_str(&article.title);
        html.push_str(r#"</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px; }
        h1 { color: #333; }
        .meta { color: #666; margin-bottom: 20px; }
        .content { color: #222; }
    </style>
</head>
<body>
    <h1>"#);
        html.push_str(&article.title);
        html.push_str(r#"</h1>
    <div class="meta">"#);

        // Add metadata
        if let Some(date) = article.published_at {
            html.push_str(&format!("Published: {}", date.format("%Y-%m-%d %H:%M:%S")));
        }
        
        if let Some(author) = &article.author {
            html.push_str(&format!(" | Author: {}", author));
        }

        html.push_str(r#"</div>
    <div class="content">"#);

        // Add content
        if let Some(content) = &article.content {
            html.push_str(content);
        } else if let Some(summary) = &article.summary {
            html.push_str(summary);
        } else {
            html.push_str("No content available.");
        }

        html.push_str(r#"</div>
</body>
</html>"#);

        html
    }
}