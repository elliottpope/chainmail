use super::{OAuthProvider, OAuthTokens};
use anyhow::{Context, Result};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use rand::Rng;

const GMAIL_CLIENT_ID: &str = "YOUR_GMAIL_CLIENT_ID.apps.googleusercontent.com";
const GMAIL_CLIENT_SECRET: &str = "YOUR_GMAIL_CLIENT_SECRET";
const GMAIL_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GMAIL_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_URL: &str = "http://localhost:8888/oauth/callback";

pub struct GmailOAuthProvider {
    client: BasicClient,
}

impl GmailOAuthProvider {
    pub fn new() -> Result<Self> {
        let client_id = ClientId::new(GMAIL_CLIENT_ID.to_string());
        let client_secret = ClientSecret::new(GMAIL_CLIENT_SECRET.to_string());
        let auth_url = AuthUrl::new(GMAIL_AUTH_URL.to_string())
            .context("Invalid authorization endpoint URL")?;
        let token_url = TokenUrl::new(GMAIL_TOKEN_URL.to_string())
            .context("Invalid token endpoint URL")?;

        let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
            .set_redirect_uri(
                RedirectUrl::new(REDIRECT_URL.to_string())
                    .context("Invalid redirect URL")?,
            );

        Ok(Self { client })
    }

    fn generate_state() -> String {
        let mut rng = rand::thread_rng();
        let state: String = (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                match idx {
                    0..=25 => (b'A' + idx) as char,
                    26..=51 => (b'a' + (idx - 26)) as char,
                    _ => (b'0' + (idx - 52)) as char,
                }
            })
            .collect();
        state
    }
}

impl OAuthProvider for GmailOAuthProvider {
    fn authorize_url(&self) -> Result<(String, String)> {
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("https://mail.google.com/".to_string()))
            .add_extra_param("access_type", "offline")
            .add_extra_param("prompt", "consent")
            .url();

        Ok((auth_url.to_string(), csrf_token.secret().to_string()))
    }

    async fn exchange_code(&self, code: &str, _state: &str) -> Result<OAuthTokens> {
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Failed to exchange authorization code for token")?;

        let access_token = token_result.access_token().secret().to_string();
        let refresh_token = token_result
            .refresh_token()
            .map(|rt| rt.secret().to_string());

        let expires_at = token_result.expires_in().map(|duration| {
            chrono::Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64)
        });

        Ok(OAuthTokens {
            access_token,
            refresh_token,
            expires_at,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let token_result = self
            .client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Failed to refresh token")?;

        let access_token = token_result.access_token().secret().to_string();
        let new_refresh_token = token_result
            .refresh_token()
            .map(|rt| rt.secret().to_string())
            .or_else(|| Some(refresh_token.to_string()));

        let expires_at = token_result.expires_in().map(|duration| {
            chrono::Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64)
        });

        Ok(OAuthTokens {
            access_token,
            refresh_token: new_refresh_token,
            expires_at,
        })
    }
}

impl Default for GmailOAuthProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create Gmail OAuth provider")
    }
}
