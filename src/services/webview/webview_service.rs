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
            // 修复：WebViewBuilder的方法链式调用，移除?操作符
            *webview = Some(
                WebViewBuilder::new()
                    .with_url("about:blank")
                    .with_html(html)
                    .with_initialization_script("document.title = 'Article Viewer';")
                    .build()?
            );
        } else if let Some(view) = webview.as_mut() {
            view.evaluate_script(&format!("document.body.innerHTML = `{}`", html))?;
        }

        Ok(())
    }