use super::types::{FetchedMessage, MailboxInfo};
use anyhow::{Context, Result};
use async_imap::Session;
use async_native_tls::{TlsConnector, TlsStream};
use chrono::{DateTime, Utc};
use futures::io::{AsyncRead, AsyncWrite};
use futures::stream::StreamExt;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

pub struct ImapClient {
    server: String,
    port: u16,
    username: String,
    password: String,
}

impl ImapClient {
    pub fn new(server: String, port: u16, username: String, password: String) -> Self {
        Self {
            server,
            port,
            username,
            password,
        }
    }

    pub async fn connect(&self) -> Result<Session<TlsStream<Compat<TcpStream>>>> {
        let address = format!("{}:{}", self.server, self.port);
        let tcp_stream = TcpStream::connect(&address)
            .await
            .context("Failed to connect to IMAP server")?
            .compat();

        let tls = TlsConnector::new();
        let tls_stream = tls
            .connect(&self.server, tcp_stream)
            .await
            .context("Failed to establish TLS connection")?;

        let client = async_imap::Client::new(tls_stream);
        let session = client
            .login(&self.username, &self.password)
            .await
            .map_err(|e| anyhow::anyhow!("Login failed: {:?}", e))?;

        Ok(session)
    }

    pub async fn list_mailboxes(&self) -> Result<Vec<String>> {
        let mut session = self.connect().await?;

        let mailboxes = session
            .list(Some(""), Some("*"))
            .await
            .context("Failed to list mailboxes")?;

        let names: Vec<String> = mailboxes
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|m| m.name().to_string())
            .collect();

        session.logout().await?;

        Ok(names)
    }

    pub async fn select_mailbox(&self, mailbox: &str) -> Result<MailboxInfo> {
        let mut session = self.connect().await?;

        let mailbox_data = session
            .select(mailbox)
            .await
            .context("Failed to select mailbox")?;

        let info = MailboxInfo {
            name: mailbox.to_string(),
            exists: mailbox_data.exists,
            recent: mailbox_data.recent,
            unseen: mailbox_data.unseen,
            uidvalidity: mailbox_data.uid_validity,
        };

        session.logout().await?;

        Ok(info)
    }

    pub async fn fetch_messages(
        &self,
        mailbox: &str,
        uid_range: Option<String>,
    ) -> Result<Vec<FetchedMessage>> {
        let mut session = self.connect().await?;

        session
            .select(mailbox)
            .await
            .context("Failed to select mailbox")?;

        let range = uid_range.unwrap_or_else(|| "1:*".to_string());

        let messages = session
            .uid_fetch(&range, "RFC822.HEADER BODY.PEEK[TEXT]<0.500> FLAGS")
            .await
            .context("Failed to fetch messages")?;

        let messages_vec: Vec<_> = messages.collect().await;

        let mut fetched_messages = Vec::new();

        for msg_result in messages_vec {
            let msg = match msg_result {
                Ok(m) => m,
                Err(_) => continue,
            };
            let uid = msg.uid.unwrap_or(0);
            let flags: Vec<String> = msg
                .flags()
                .map(|f| format!("{:?}", f))
                .collect();

            let seen = msg.flags().any(|f| matches!(f, async_imap::types::Flag::Seen));
            let flagged = msg.flags().any(|f| matches!(f, async_imap::types::Flag::Flagged));

            let headers = msg.header();
            let body_bytes = msg.body();

            let (message_id, subject, from, to, date) = if let Some(header_bytes) = headers {
                Self::parse_headers(header_bytes)
            } else {
                (None, None, None, None, None)
            };

            let body_preview = body_bytes
                .and_then(|b| String::from_utf8(b.to_vec()).ok())
                .map(|s| {
                    s.lines()
                        .take(5)
                        .collect::<Vec<_>>()
                        .join("\n")
                        .chars()
                        .take(500)
                        .collect()
                });

            fetched_messages.push(FetchedMessage {
                uid,
                message_id,
                subject,
                from,
                to,
                date,
                body_preview,
                flags,
                seen,
                flagged,
            });
        }

        session.logout().await?;

        Ok(fetched_messages)
    }

    fn parse_headers(header_bytes: &[u8]) -> (Option<String>, Option<String>, Option<String>, Option<String>, Option<DateTime<Utc>>) {
        let header_str = String::from_utf8_lossy(header_bytes);
        let mut headers = HashMap::new();

        for line in header_str.lines() {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        let message_id = headers.get("message-id").cloned();
        let subject = headers.get("subject").cloned();
        let from = headers.get("from").cloned();
        let to = headers.get("to").cloned();

        let date = headers.get("date").and_then(|d| {
            chrono::DateTime::parse_from_rfc2822(d)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        (message_id, subject, from, to, date)
    }

    pub async fn mark_as_seen(&self, mailbox: &str, uid: u32) -> Result<()> {
        let mut session = self.connect().await?;

        session
            .select(mailbox)
            .await
            .context("Failed to select mailbox")?;

        session
            .uid_store(format!("{}", uid), "+FLAGS (\\Seen)")
            .await
            .context("Failed to mark message as seen")?;

        session.logout().await?;

        Ok(())
    }

    pub async fn delete_message(&self, mailbox: &str, uid: u32) -> Result<()> {
        let mut session = self.connect().await?;

        session
            .select(mailbox)
            .await
            .context("Failed to select mailbox")?;

        session
            .uid_store(format!("{}", uid), "+FLAGS (\\Deleted)")
            .await
            .context("Failed to mark message as deleted")?;

        session
            .expunge()
            .await
            .context("Failed to expunge deleted messages")?;

        session.logout().await?;

        Ok(())
    }
}
