# Chainmail

A desktop email client for Linux/Wayland written in Rust with Vim-like keybindings and motions.

## Features

- **Vim-inspired interface**: Navigate and interact using familiar Vim keybindings
- **Four modes**: Normal, Insert, Visual, and Command modes for different interactions
- **IMAP support**: Connect to IMAP email servers (POP and SMTP coming later)
- **Multiple accounts**: Manage multiple email accounts in one interface
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
│   └── layout.rs     # UI layout (account selector, email list, email display)
├── vim/              # Vim mode system
│   ├── mod.rs
│   ├── modes.rs      # Mode definitions (Normal, Insert, Visual, Command)
│   └── keybindings.rs # Keyboard event handling and actions
├── db/               # Database layer
│   ├── mod.rs
│   ├── schema.rs     # Database schema creation
│   ├── models.rs     # Data models (Account, Mailbox, Message)
│   └── queries.rs    # Database queries
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
- `:quit` or `:q` - Quit application
- `Backspace` - Delete last character (returns to Normal mode if empty)
- `Enter` - Execute command
- `Esc` - Cancel and return to Normal mode

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
- **chrono** - Date and time handling
- **serde** - Serialization framework

## Current Limitations

- No POP or SMTP support yet (IMAP only)
- Account management UI not yet implemented (accounts must be added via database)
- Email composition not yet implemented
- No attachment support yet
- Search is local only (no server-side IMAP search)

## Roadmap

- [ ] Add account management UI
- [ ] Implement email composition (Insert mode)
- [ ] Add SMTP support for sending emails
- [ ] Implement attachment handling
- [ ] Add server-side IMAP search
- [ ] Support for more complex Vim motions
- [ ] Customizable keybindings
- [ ] Themes and styling options
- [ ] Email filtering and rules
- [ ] Offline mode improvements

## License

MIT License (see LICENSE file)
