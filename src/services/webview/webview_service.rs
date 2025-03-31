use std::sync::{Arc, Mutex};
use anyhow::Result;
use wry::{WebView, WebViewBuilder};

pub struct WebViewService {
    webview: Arc<Mutex<Option<WebView>>>,
}

impl WebViewService {
    pub fn new() -> Self {
        Self {
            webview: Arc::new(Mutex::new(None)),
        }
    }

    pub fn show_content(&mut self, content: &str) -> Result<()> {
        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <style>
                    body {{
                        font-family: system-ui, -apple-system, sans-serif;
                        line-height: 1.6;
                        padding: 20px;
                        margin: 0;
                        color: #333;
                        background: #fff;
                    }}
                    img {{
                        max-width: 100%;
                        height: auto;
                    }}
                    pre {{
                        background: #f5f5f5;
                        padding: 15px;
                        border-radius: 5px;
                        overflow-x: auto;
                    }}
                </style>
            </head>
            <body>
                {}
            </body>
            </html>
            "#,
            content
        );

        let mut webview = self.webview.lock().unwrap();
        if webview.is_none() {
            *webview = Some(
                WebViewBuilder::new()
                    .with_title("Article Viewer")
                    .with_html(html)?
                    .build()?
            );
        } else if let Some(view) = webview.as_mut() {
            view.evaluate_script(&format!("document.body.innerHTML = `{}`", html))?;
        }

        Ok(())
    }

    pub fn hide(&mut self) {
        let mut webview = self.webview.lock().unwrap();
        *webview = None;
    }

    pub fn is_visible(&self) -> bool {
        self.webview.lock().unwrap().is_some()
    }
}