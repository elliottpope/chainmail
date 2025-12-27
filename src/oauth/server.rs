use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use tiny_http::{Response, Server};
use url::Url;

#[derive(Debug, Clone)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

pub struct OAuthCallbackServer {
    server: Arc<Server>,
    callback: Arc<Mutex<Option<OAuthCallback>>>,
}

impl OAuthCallbackServer {
    pub fn new(port: u16) -> Result<Self> {
        let server = Server::http(format!("127.0.0.1:{}", port))
            .map_err(|e| anyhow::anyhow!("Failed to start OAuth callback server: {}", e))?;

        Ok(Self {
            server: Arc::new(server),
            callback: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn wait_for_callback(&self) -> Result<OAuthCallback> {
        let server = self.server.clone();
        let callback = self.callback.clone();

        tokio::task::spawn_blocking(move || {
            for request in server.incoming_requests() {
                let url_str = format!("http://localhost{}", request.url());
                let url = Url::parse(&url_str)?;

                let mut code = None;
                let mut state = None;

                for (key, value) in url.query_pairs() {
                    match key.as_ref() {
                        "code" => code = Some(value.to_string()),
                        "state" => state = Some(value.to_string()),
                        _ => {}
                    }
                }

                let html_response = if let (Some(code_val), Some(state_val)) = (code, state) {
                    let oauth_callback = OAuthCallback {
                        code: code_val,
                        state: state_val,
                    };

                    *callback.lock().unwrap() = Some(oauth_callback.clone());

                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Authorization Successful</title>
                        <style>
                            body {
                                font-family: Arial, sans-serif;
                                display: flex;
                                justify-content: center;
                                align-items: center;
                                height: 100vh;
                                margin: 0;
                                background-color: #f0f0f0;
                            }
                            .container {
                                text-align: center;
                                background-color: white;
                                padding: 40px;
                                border-radius: 10px;
                                box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                            }
                            h1 { color: #4CAF50; }
                            p { color: #666; }
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <h1>✓ Authorization Successful</h1>
                            <p>You can now close this window and return to Chainmail.</p>
                        </div>
                    </body>
                    </html>
                    "#
                } else {
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Authorization Failed</title>
                        <style>
                            body {
                                font-family: Arial, sans-serif;
                                display: flex;
                                justify-content: center;
                                align-items: center;
                                height: 100vh;
                                margin: 0;
                                background-color: #f0f0f0;
                            }
                            .container {
                                text-align: center;
                                background-color: white;
                                padding: 40px;
                                border-radius: 10px;
                                box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                            }
                            h1 { color: #f44336; }
                            p { color: #666; }
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <h1>✗ Authorization Failed</h1>
                            <p>Missing authorization code. Please try again.</p>
                        </div>
                    </body>
                    </html>
                    "#
                };

                let response = Response::from_string(html_response)
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                            .unwrap(),
                    );

                request.respond(response)?;

                if callback.lock().unwrap().is_some() {
                    break;
                }
            }

            callback
                .lock()
                .unwrap()
                .clone()
                .context("No callback received")
        })
        .await?
    }
}

pub fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .context("Failed to open browser with xdg-open")?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .context("Failed to open browser")?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", url])
            .spawn()
            .context("Failed to open browser")?;
    }

    Ok(())
}
