# Chainmail

A desktop email client for Linux/Wayland written in Rust with Vim-like keybindings and motions.

## Features

- **Vim-inspired interface**: Navigate and interact using familiar Vim keybindings
- **Four modes**: Normal, Insert, Visual, and Command modes for different interactions
- **IMAP support**: Connect to IMAP email servers (POP and SMTP coming later)
- **Multiple accounts**: Manage multiple email accounts in one interface
- **OAuth 2.0 support**: Secure Gmail authentication via OAuth
- **SQLite caching**: Fast local caching of emails and account information
- **Search functionality**: Quickly find emails with the built-in search
- **Modern GUI**: Built with iced for a native, performant interface

## Project Structure

```
src/
├── main.rs           # Application entry point
├── app.rs            # Main application state and logic
├── ui/               # UI components
│   ├── mod.rs
│   ├── layout.rs     # Main email view layout
│   └── account_management.rs # Account management screens
├── vim/              # Vim mode system
│   ├── mod.rs
│   ├── modes.rs      # Mode definitions (Normal, Insert, Visual, Command)
│   └── keybindings.rs # Keyboard event handling and actions
├── db/               # Database layer
│   ├── mod.rs
│   ├── schema.rs     # Database schema creation
│   ├── models.rs     # Data models (Account, Mailbox, Message)
│   └── queries.rs    # Database queries
├── oauth/            # OAuth 2.0 authentication
│   ├── mod.rs
│   ├── gmail.rs      # Gmail OAuth provider
│   └── server.rs     # Local OAuth callback server
└── imap/             # IMAP client
    ├── mod.rs
    ├── client.rs     # IMAP connection manager
    └── types.rs      # IMAP-related types
```

## UI Layout

```
┌─────────────────────────────────────────────────────────┐
│ Search Bar (/ to search, : for commands)               │
├──────────┬────────────────┬─────────────────────────────┤
│ Account  │ Email List     │ Email Display               │
│ Selector │                │                             │
│          │ ● Subject 1    │ Subject: Test Email         │
│ All      │   From: user   │ From: sender@example.com    │
│ Inboxes  │   Date: ...    │ To: you@example.com         │
│          │                │                             │
│          │   Subject 2    │ Email body preview...       │
│          │   From: ...    │                             │
│          │                │                             │
├──────────┴────────────────┴─────────────────────────────┤
│ Status: NORMAL                                          │
└─────────────────────────────────────────────────────────┘
```

## Vim Keybindings

### Normal Mode

- `j` / `k` - Move down/up in email list
- `h` / `l` - Move left/right (future use)
- `gg` - Go to top of list
- `G` - Go to bottom of list
- `Ctrl+d` / `Ctrl+u` - Page down/up
- `Enter` - Open selected email
- `*` - Toggle flagged/starred
- `r` - Mark as read
- `u` - Mark as unread
- `dd` - Delete selected email
- `yy` - Yank (copy) selected email
- `i` - Enter Insert mode
- `v` - Enter Visual mode
- `:` - Enter Command mode
- `/` - Quick search (enters command mode with "find ")
- `0-9` - Count prefix (e.g., `5j` moves down 5 emails)

### Visual Mode

- `j` / `k` - Extend selection down/up
- `gg` / `G` - Extend selection to top/bottom
- `d` - Delete selected emails
- `y` - Yank selected emails
- `*` - Toggle flagged on all selected
- `Esc` - Return to Normal mode

### Insert Mode

- Used for composing emails (to be implemented)
- `Esc` - Return to Normal mode

### Command Mode

- `:find <query>` or `:f <query>` - Search for emails
- `:account` or `:acc` - Open account management
- `:quit` or `:q` - Quit application
- `Backspace` - Delete last character (returns to Normal mode if empty)
- `Enter` - Execute command
- `Esc` - Cancel and return to Normal mode

## Adding Accounts

### Gmail with OAuth 2.0

Chainmail supports Gmail accounts using secure OAuth 2.0 authentication:

1. Enter command mode with `:`
2. Type `account` or `acc` to open account management
3. Click "Add Account"
4. Select "Gmail" as the provider
5. Your browser will open for Gmail authorization
6. Sign in to your Gmail account
7. Review and accept the permissions
8. You'll be automatically redirected back to Chainmail
9. Your account will be added and ready to use

**Note**: For Gmail OAuth to work, you need to configure OAuth credentials in `src/oauth/gmail.rs`:
- Set `GMAIL_CLIENT_ID` to your Google Cloud Console client ID
- Set `GMAIL_CLIENT_SECRET` to your client secret

See [Google's OAuth 2.0 documentation](https://developers.google.com/identity/protocols/oauth2) for details on creating OAuth credentials.

### Other IMAP Providers

Support for manual IMAP configuration is coming soon. For now, only Gmail with OAuth is supported.

## Building and Running

### Prerequisites

- Rust 1.70 or later
- SQLite development libraries
- OpenSSL development libraries
- Wayland development libraries (for Linux/Wayland)

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run
```

## Database Schema

The application uses SQLite for local storage with three main tables:

### Accounts
Stores IMAP account credentials and settings.

### Mailboxes
Stores mailbox information for each account (INBOX, Sent, etc.)

### Messages
Caches email messages with metadata and body previews.

## Dependencies

- **iced** - Cross-platform GUI framework
- **tokio** - Async runtime
- **sqlx** - Async SQL database access
- **async-imap** - IMAP client
- **async-native-tls** - TLS support for IMAP
- **oauth2** - OAuth 2.0 client library
- **reqwest** - HTTP client for OAuth token exchange
- **tiny_http** - Lightweight HTTP server for OAuth callback
- **chrono** - Date and time handling
- **serde** - Serialization framework

## Current Limitations

- No POP or SMTP support yet (IMAP only)
- Gmail OAuth requires manual configuration of client ID/secret
- Manual IMAP account setup not yet implemented (only OAuth providers)
- Email composition not yet implemented
- No attachment support yet
- Search is local only (no server-side IMAP search)
- No automatic sync/fetching of emails yet

## Roadmap

- [x] Add account management UI
- [x] Implement Gmail OAuth 2.0 authentication
- [ ] Add manual IMAP configuration for other providers
- [ ] Implement email composition (Insert mode)
- [ ] Add SMTP support for sending emails
- [ ] Implement automatic email syncing/fetching
- [ ] Implement attachment handling
- [ ] Add server-side IMAP search
- [ ] Support OAuth for other providers (Outlook, Yahoo, etc.)
- [ ] Support for more complex Vim motions
- [ ] Customizable keybindings
- [ ] Themes and styling options
- [ ] Email filtering and rules
- [ ] Offline mode improvements

## License

MIT License (see LICENSE file)
