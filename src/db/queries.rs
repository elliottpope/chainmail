use super::models::{Account, Mailbox, Message, MessageWithMailbox};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

pub async fn insert_account(
    pool: &SqlitePool,
    name: &str,
    email: &str,
    imap_server: &str,
    imap_port: u16,
    username: &str,
    password: &str,
) -> Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO accounts (name, email, imap_server, imap_port, username, password)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(name)
    .bind(email)
    .bind(imap_server)
    .bind(imap_port as i64)
    .bind(username)
    .bind(password)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_all_accounts(pool: &SqlitePool) -> Result<Vec<Account>> {
    let accounts = sqlx::query_as::<_, Account>("SELECT * FROM accounts ORDER BY name")
        .fetch_all(pool)
        .await?;

    Ok(accounts)
}

pub async fn get_account(pool: &SqlitePool, account_id: i64) -> Result<Option<Account>> {
    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;

    Ok(account)
}

pub async fn insert_mailbox(
    pool: &SqlitePool,
    account_id: i64,
    name: &str,
    uidvalidity: Option<i64>,
) -> Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO mailboxes (account_id, name, uidvalidity)
        VALUES (?, ?, ?)
        ON CONFLICT(account_id, name) DO UPDATE SET uidvalidity = excluded.uidvalidity
        "#,
    )
    .bind(account_id)
    .bind(name)
    .bind(uidvalidity)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_mailboxes_for_account(pool: &SqlitePool, account_id: i64) -> Result<Vec<Mailbox>> {
    let mailboxes = sqlx::query_as::<_, Mailbox>(
        "SELECT * FROM mailboxes WHERE account_id = ? ORDER BY name",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    Ok(mailboxes)
}

pub async fn insert_message(
    pool: &SqlitePool,
    mailbox_id: i64,
    uid: i64,
    message_id: Option<&str>,
    subject: Option<&str>,
    from_addr: Option<&str>,
    to_addr: Option<&str>,
    date: Option<chrono::DateTime<chrono::Utc>>,
    body_preview: Option<&str>,
    flags: &str,
    seen: bool,
    flagged: bool,
) -> Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO messages (mailbox_id, uid, message_id, subject, from_addr, to_addr, date, body_preview, flags, seen, flagged)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(mailbox_id, uid) DO UPDATE SET
            subject = excluded.subject,
            from_addr = excluded.from_addr,
            to_addr = excluded.to_addr,
            date = excluded.date,
            body_preview = excluded.body_preview,
            flags = excluded.flags,
            seen = excluded.seen,
            flagged = excluded.flagged
        "#,
    )
    .bind(mailbox_id)
    .bind(uid)
    .bind(message_id)
    .bind(subject)
    .bind(from_addr)
    .bind(to_addr)
    .bind(date)
    .bind(body_preview)
    .bind(flags)
    .bind(seen as i64)
    .bind(flagged as i64)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_all_messages(pool: &SqlitePool) -> Result<Vec<MessageWithMailbox>> {
    let rows = sqlx::query(
        r#"
        SELECT m.id, m.mailbox_id, m.uid, m.message_id, m.subject, m.from_addr, m.to_addr,
               m.date, m.body_preview, m.flags, m.seen, m.flagged, m.created_at,
               mb.name as mailbox_name, a.name as account_name, a.email as account_email
        FROM messages m
        JOIN mailboxes mb ON m.mailbox_id = mb.id
        JOIN accounts a ON mb.account_id = a.id
        ORDER BY m.date DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let messages = rows
        .into_iter()
        .map(|row| {
            let message = Message {
                id: row.get("id"),
                mailbox_id: row.get("mailbox_id"),
                uid: row.get("uid"),
                message_id: row.get("message_id"),
                subject: row.get("subject"),
                from_addr: row.get("from_addr"),
                to_addr: row.get("to_addr"),
                date: row.get("date"),
                body_preview: row.get("body_preview"),
                flags: row.get("flags"),
                seen: row.get::<i64, _>("seen") != 0,
                flagged: row.get::<i64, _>("flagged") != 0,
                created_at: row.get("created_at"),
            };
            MessageWithMailbox {
                message,
                mailbox_name: row.get("mailbox_name"),
                account_name: row.get("account_name"),
                account_email: row.get("account_email"),
            }
        })
        .collect();

    Ok(messages)
}

pub async fn get_messages_for_mailbox(pool: &SqlitePool, mailbox_id: i64) -> Result<Vec<Message>> {
    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE mailbox_id = ? ORDER BY date DESC",
    )
    .bind(mailbox_id)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn search_messages(pool: &SqlitePool, query: &str) -> Result<Vec<MessageWithMailbox>> {
    let search_pattern = format!("%{}%", query);
    let rows = sqlx::query(
        r#"
        SELECT m.id, m.mailbox_id, m.uid, m.message_id, m.subject, m.from_addr, m.to_addr,
               m.date, m.body_preview, m.flags, m.seen, m.flagged, m.created_at,
               mb.name as mailbox_name, a.name as account_name, a.email as account_email
        FROM messages m
        JOIN mailboxes mb ON m.mailbox_id = mb.id
        JOIN accounts a ON mb.account_id = a.id
        WHERE m.subject LIKE ? OR m.from_addr LIKE ? OR m.body_preview LIKE ?
        ORDER BY m.date DESC
        "#,
    )
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_all(pool)
    .await?;

    let messages = rows
        .into_iter()
        .map(|row| {
            let message = Message {
                id: row.get("id"),
                mailbox_id: row.get("mailbox_id"),
                uid: row.get("uid"),
                message_id: row.get("message_id"),
                subject: row.get("subject"),
                from_addr: row.get("from_addr"),
                to_addr: row.get("to_addr"),
                date: row.get("date"),
                body_preview: row.get("body_preview"),
                flags: row.get("flags"),
                seen: row.get::<i64, _>("seen") != 0,
                flagged: row.get::<i64, _>("flagged") != 0,
                created_at: row.get("created_at"),
            };
            MessageWithMailbox {
                message,
                mailbox_name: row.get("mailbox_name"),
                account_name: row.get("account_name"),
                account_email: row.get("account_email"),
            }
        })
        .collect();

    Ok(messages)
}
