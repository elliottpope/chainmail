use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub imap_server: String,
    pub imap_port: u16,
    pub username: String,
    pub password: Option<String>,
    pub auth_type: String,
    pub oauth_access_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub oauth_expires_at: Option<DateTime<Utc>>,
    pub provider: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthType {
    Password,
    OAuth,
}

impl AuthType {
    pub fn to_string(&self) -> &'static str {
        match self {
            AuthType::Password => "password",
            AuthType::OAuth => "oauth",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "oauth" => AuthType::OAuth,
            _ => AuthType::Password,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mailbox {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub uidvalidity: Option<i64>,
    pub last_synced: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i64,
    pub mailbox_id: i64,
    pub uid: i64,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from_addr: Option<String>,
    pub to_addr: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub body_preview: Option<String>,
    pub flags: String,
    pub seen: bool,
    pub flagged: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MessageWithMailbox {
    pub message: Message,
    pub mailbox_name: String,
    pub account_name: String,
    pub account_email: String,
}
