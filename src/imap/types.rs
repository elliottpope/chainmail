use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct MailboxInfo {
    pub name: String,
    pub exists: u32,
    pub recent: u32,
    pub unseen: Option<u32>,
    pub uidvalidity: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct FetchedMessage {
    pub uid: u32,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub body_preview: Option<String>,
    pub flags: Vec<String>,
    pub seen: bool,
    pub flagged: bool,
}
