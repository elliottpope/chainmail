pub mod gmail;
pub mod server;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountProvider {
    Gmail,
    Other,
}

impl AccountProvider {
    pub fn to_string(&self) -> &'static str {
        match self {
            AccountProvider::Gmail => "Gmail",
            AccountProvider::Other => "Other",
        }
    }
}

pub trait OAuthProvider {
    fn authorize_url(&self) -> Result<(String, String)>;
    fn exchange_code(&self, code: &str, state: &str) -> impl std::future::Future<Output = Result<OAuthTokens>> + Send;
    fn refresh_token(&self, refresh_token: &str) -> impl std::future::Future<Output = Result<OAuthTokens>> + Send;
}
