use std::sync::{Arc, Mutex};
use anyhow::Result;
use wry::{WebView, WebViewBuilder};
use tao::{
    window::{Window, WindowBuilder},
    dpi::LogicalSize,
    event_loop::{EventLoop, ControlFlow},
};

/// Container for WebView and its associated Window
struct WebViewData {
    webview: WebView,
    window: Window,
    _event_loop: EventLoop<()>, // Store event loop to keep it alive
}

/// Service for displaying web content in a native window
pub struct WebViewService {
    // Store the webview data
    webview_data: Arc<Mutex<Option<WebViewData>>>,
}

impl WebViewService {
    /// Creates a new WebViewService instance
    pub fn new() -> Self {
        Self {
            webview_data: Arc::new(Mutex::new(None)),
        }
    }

    /// Displays HTML content in a webview
    pub fn show_content(&mut self, content: &str) -> Result<(), anyhow::Error> {
        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset=\"UTF-8\">
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
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

        let mut webview_data = self.webview_data.lock().unwrap();
        
        if webview_data.is_none() {
            // Create a new event loop (required for window creation)
            let event_loop = EventLoop::new();
            
            // Create a window with tao
            let window = WindowBuilder::new()
                .with_title("Article Viewer")
                .with_inner_size(LogicalSize::new(800.0, 600.0))
                .build(&event_loop)
                .map_err(|e| anyhow::anyhow!("Failed to build window: {}", e))?;
                
            // Now create the webview with the window reference
            let webview = WebViewBuilder::new()
                .with_url("about:blank")
                .with_html(&html)
                .with_initialization_script("document.title = 'Article Viewer';")
                // Pass window reference to satisfy HasWindowHandle trait
                .build(&window)
                .map_err(|e| anyhow::anyhow!("Failed to build WebView: {}", e))?;
                
            // Store both the window and webview together
            // We also store the event_loop to keep it alive
            *webview_data = Some(WebViewData { 
                webview, 
                window,
                _event_loop: event_loop,
            });
            
            // In a real application, you would want to run the event loop
            // Since we're in a library function, we won't block here
            // Instead we'll keep the event loop alive and assume the application
            // has its own event handling mechanism
        } else if let Some(data) = webview_data.as_mut() {
            // Update content in existing webview
            data.webview.evaluate_script(&format!("document.body.innerHTML = `{}`", html))
                .map_err(|e| anyhow::anyhow!("Failed to update content: {}", e))?;
        }

        Ok(())
    }

    /// Hides the webview by destroying the window and webview
    pub fn hide(&mut self) {
        let mut webview_data = self.webview_data.lock().unwrap();
        *webview_data = None;
    }

    /// Checks if the webview is currently visible
    pub fn is_visible(&self) -> bool {
        self.webview_data.lock().unwrap().is_some()
    }
}