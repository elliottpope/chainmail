use crate::db::{models::MessageWithMailbox, Database};
use crate::oauth::{
    gmail::GmailOAuthProvider, server::OAuthCallbackServer, AccountProvider, OAuthProvider,
    OAuthTokens,
};
use crate::vim::{VimAction, VimKeyHandler, VimState};
use iced::keyboard::{self, Key, Modifiers};
use iced::{event, Element, Event, Subscription, Task, Theme};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
    KeyPressed(Key, Modifiers),
    SelectAccount(usize),
    SelectMessage(usize),
    LoadMessages(Vec<MessageWithMailbox>),
    ExecuteVimAction(VimAction),
    SearchQueryChanged(String),
    ShowAccountManagement,
    ShowProviderSelection,
    SelectProvider(AccountProvider),
    StartOAuthFlow,
    OAuthComplete(Result<OAuthTokens, String>),
    SaveOAuthAccount(String, String),
    BackToMain,
    AccountNameInput(String),
    EmailInput(String),
    MenuNavigateUp,
    MenuNavigateDown,
    MenuSelect,
    MenuGoBack,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    Main,
    AccountManagement,
    ProviderSelection,
    OAuthInProgress,
    ManualAccountSetup,
}

pub struct ChainmailApp {
    pub db: Arc<Mutex<Database>>,
    vim_state: VimState,
    selected_account: Option<usize>,
    selected_message: Option<usize>,
    messages: Vec<MessageWithMailbox>,
    displayed_messages: Vec<MessageWithMailbox>,
    search_query: String,
    visual_selection_start: Option<usize>,
    current_screen: AppScreen,
    selected_provider: Option<AccountProvider>,
    account_name_input: String,
    email_input: String,
    oauth_state: Option<String>,
    menu_selection_index: usize,
}

impl ChainmailApp {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let db_path = "sqlite://chainmail.db";
        let db = Database::new(db_path).await?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            vim_state: VimState::new(),
            selected_account: None,
            selected_message: None,
            messages: Vec::new(),
            displayed_messages: Vec::new(),
            search_query: String::new(),
            visual_selection_start: None,
            current_screen: AppScreen::Main,
            selected_provider: None,
            account_name_input: String::new(),
            email_input: String::new(),
            oauth_state: None,
            menu_selection_index: 0,
        })
    }

    fn handle_vim_action(&mut self, action: VimAction) -> Task<Message> {
        match action {
            VimAction::MoveDown(count) => {
                if let Some(current) = self.selected_message {
                    let new_index =
                        (current + count).min(self.displayed_messages.len().saturating_sub(1));
                    self.selected_message = Some(new_index);
                }
                Task::none()
            }
            VimAction::MoveUp(count) => {
                if let Some(current) = self.selected_message {
                    let new_index = current.saturating_sub(count);
                    self.selected_message = Some(new_index);
                }
                Task::none()
            }
            VimAction::MoveToTop => {
                self.selected_message = Some(0);
                Task::none()
            }
            VimAction::MoveToBottom => {
                if !self.displayed_messages.is_empty() {
                    self.selected_message = Some(self.displayed_messages.len() - 1);
                }
                Task::none()
            }
            VimAction::EnterVisualMode => {
                self.visual_selection_start = self.selected_message;
                Task::none()
            }
            VimAction::EnterNormalMode => {
                self.visual_selection_start = None;
                Task::none()
            }
            VimAction::ExecuteCommand(cmd) => {
                self.execute_command(&cmd);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn execute_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "find" | "f" => {
                if parts.len() > 1 {
                    let query = parts[1..].join(" ");
                    self.search_query = query.clone();
                    self.filter_messages();
                }
            }
            "account" | "acc" | "accounts" => {
                self.current_screen = AppScreen::AccountManagement;
            }
            "quit" | "q" => {
                std::process::exit(0);
            }
            _ => {}
        }
    }

    fn filter_messages(&mut self) {
        if self.search_query.is_empty() {
            self.displayed_messages = self.messages.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.displayed_messages = self
                .messages
                .iter()
                .filter(|msg| {
                    msg.message
                        .subject
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                        || msg
                            .message
                            .from_addr
                            .as_ref()
                            .map(|s| s.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                })
                .cloned()
                .collect();
        }

        if !self.displayed_messages.is_empty() {
            self.selected_message = Some(0);
        } else {
            self.selected_message = None;
        }
    }

    pub async fn load_all_messages(
        db: Arc<Mutex<Database>>,
    ) -> Result<Vec<MessageWithMailbox>, anyhow::Error> {
        let db = db.lock().await;
        let messages = crate::db::queries::get_all_messages(db.pool()).await?;
        Ok(messages)
    }
}

impl ChainmailApp {
    pub fn title(&self) -> String {
        String::from("Chainmail - Vim Email Client")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EventOccurred(event) => {
                if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                    return Task::done(Message::KeyPressed(key, modifiers));
                }
                Task::none()
            }
            Message::KeyPressed(key, modifiers) => {
                // If we're in account management screens, handle menu navigation
                if self.current_screen != AppScreen::Main {
                    return self.handle_menu_keyboard(&key, &modifiers);
                }

                // Otherwise, handle Vim keybindings
                let action = VimKeyHandler::handle_key_press(&mut self.vim_state, &key, &modifiers);
                Task::done(Message::ExecuteVimAction(action))
            }
            Message::ExecuteVimAction(action) => self.handle_vim_action(action),
            Message::SelectAccount(index) => {
                self.selected_account = Some(index);
                Task::none()
            }
            Message::SelectMessage(index) => {
                self.selected_message = Some(index);
                Task::none()
            }
            Message::LoadMessages(messages) => {
                self.messages = messages;
                self.displayed_messages = self.messages.clone();
                if !self.displayed_messages.is_empty() && self.selected_message.is_none() {
                    self.selected_message = Some(0);
                }
                Task::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                self.filter_messages();
                Task::none()
            }
            Message::ShowAccountManagement => {
                self.current_screen = AppScreen::AccountManagement;
                Task::none()
            }
            Message::ShowProviderSelection => {
                self.current_screen = AppScreen::ProviderSelection;
                Task::none()
            }
            Message::SelectProvider(provider) => {
                self.selected_provider = Some(provider.clone());
                if provider == AccountProvider::Gmail {
                    Task::done(Message::StartOAuthFlow)
                } else {
                    self.current_screen = AppScreen::ManualAccountSetup;
                    Task::none()
                }
            }
            Message::StartOAuthFlow => {
                let provider = GmailOAuthProvider::new().expect("Failed to create OAuth provider");

                match provider.authorize_url() {
                    Ok((auth_url, state)) => {
                        self.oauth_state = Some(state.clone());
                        self.current_screen = AppScreen::OAuthInProgress;

                        if let Err(e) = crate::oauth::server::open_browser(&auth_url) {
                            tracing::error!("Failed to open browser: {}", e);
                            return Task::done(Message::OAuthComplete(Err(format!(
                                "Failed to open browser: {}",
                                e
                            ))));
                        }

                        let oauth_state_clone = state.clone();
                        Task::perform(
                            async move {
                                let server = OAuthCallbackServer::new(8888)
                                    .expect("Failed to start OAuth callback server");

                                match server.wait_for_callback().await {
                                    Ok(callback) => {
                                        if callback.state != oauth_state_clone {
                                            return Err("State mismatch".to_string());
                                        }

                                        let provider = GmailOAuthProvider::new()
                                            .expect("Failed to create OAuth provider");

                                        match provider
                                            .exchange_code(&callback.code, &callback.state)
                                            .await
                                        {
                                            Ok(tokens) => Ok(tokens),
                                            Err(e) => Err(format!("Token exchange failed: {}", e)),
                                        }
                                    }
                                    Err(e) => Err(format!("OAuth callback failed: {}", e)),
                                }
                            },
                            Message::OAuthComplete,
                        )
                    }
                    Err(e) => {
                        tracing::error!("Failed to generate auth URL: {}", e);
                        Task::done(Message::OAuthComplete(Err(format!(
                            "Failed to generate auth URL: {}",
                            e
                        ))))
                    }
                }
            }
            Message::OAuthComplete(result) => match result {
                Ok(tokens) => {
                    let db = self.db.clone();
                    let email = self.email_input.clone();
                    let name = self.account_name_input.clone();

                    Task::perform(
                        async move {
                            let db = db.lock().await;
                            let account_name = if name.is_empty() {
                                email.split('@').next().unwrap_or(&email).to_string()
                            } else {
                                name
                            };

                            match crate::db::queries::insert_oauth_account(
                                db.pool(),
                                &account_name,
                                &email,
                                "imap.gmail.com",
                                993,
                                &email,
                                "gmail",
                                &tokens.access_token,
                                tokens.refresh_token.as_deref(),
                                tokens.expires_at,
                            )
                            .await
                            {
                                Ok(_) => (account_name, email),
                                Err(e) => {
                                    tracing::error!("Failed to save OAuth account: {}", e);
                                    (String::new(), String::new())
                                }
                            }
                        },
                        |(name, email)| Message::SaveOAuthAccount(name, email),
                    )
                }
                Err(e) => {
                    tracing::error!("OAuth flow failed: {}", e);
                    self.current_screen = AppScreen::AccountManagement;
                    Task::none()
                }
            },
            Message::SaveOAuthAccount(name, email) => {
                if !name.is_empty() && !email.is_empty() {
                    tracing::info!("Successfully added OAuth account: {} ({})", name, email);
                }
                self.current_screen = AppScreen::Main;
                self.account_name_input.clear();
                self.email_input.clear();
                self.selected_provider = None;
                self.oauth_state = None;
                Task::none()
            }
            Message::BackToMain => {
                self.current_screen = AppScreen::Main;
                self.account_name_input.clear();
                self.email_input.clear();
                self.selected_provider = None;
                self.oauth_state = None;
                Task::none()
            }
            Message::AccountNameInput(input) => {
                self.account_name_input = input;
                Task::none()
            }
            Message::EmailInput(input) => {
                self.email_input = input;
                Task::none()
            }
            Message::MenuNavigateUp => {
                if self.menu_selection_index > 0 {
                    self.menu_selection_index -= 1;
                }
                Task::none()
            }
            Message::MenuNavigateDown => {
                let max_index = self.get_menu_option_count();
                if self.menu_selection_index < max_index - 1 {
                    self.menu_selection_index += 1;
                }
                Task::none()
            }
            Message::MenuSelect => self.handle_menu_selection(),
            Message::MenuGoBack => self.handle_menu_go_back(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        crate::ui::view(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn selected_account(&self) -> Option<usize> {
        self.selected_account
    }

    pub fn selected_message(&self) -> Option<usize> {
        self.selected_message
    }

    pub fn messages(&self) -> &[MessageWithMailbox] {
        &self.displayed_messages
    }

    pub fn vim_state(&self) -> &VimState {
        &self.vim_state
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn visual_selection_range(&self) -> Option<(usize, usize)> {
        if let Some(start) = self.visual_selection_start {
            if let Some(current) = self.selected_message {
                let min = start.min(current);
                let max = start.max(current);
                return Some((min, max));
            }
        }
        None
    }

    pub fn current_screen(&self) -> &AppScreen {
        &self.current_screen
    }

    pub fn selected_provider(&self) -> Option<&AccountProvider> {
        self.selected_provider.as_ref()
    }

    pub fn account_name_input(&self) -> &str {
        &self.account_name_input
    }

    pub fn email_input(&self) -> &str {
        &self.email_input
    }

    pub fn menu_selection_index(&self) -> usize {
        self.menu_selection_index
    }

    fn handle_menu_keyboard(&self, key: &Key, _modifiers: &Modifiers) -> Task<Message> {
        match key {
            Key::Named(keyboard::key::Named::ArrowUp) => Task::done(Message::MenuNavigateUp),
            Key::Named(keyboard::key::Named::ArrowDown) => Task::done(Message::MenuNavigateDown),
            Key::Named(keyboard::key::Named::Enter) => Task::done(Message::MenuSelect),
            Key::Named(keyboard::key::Named::Escape) => Task::done(Message::MenuGoBack),
            Key::Character(c) if c == "j" => Task::done(Message::MenuNavigateDown),
            Key::Character(c) if c == "k" => Task::done(Message::MenuNavigateUp),
            _ => Task::none(),
        }
    }

    fn get_menu_option_count(&self) -> usize {
        match self.current_screen {
            AppScreen::AccountManagement => 2, // "Add Account" and "Back to Mail"
            AppScreen::ProviderSelection => 3, // "Gmail", "Other", "Back"
            AppScreen::OAuthInProgress => 1,   // "Cancel"
            AppScreen::ManualAccountSetup => 1, // "Back"
            AppScreen::Main => 0,
        }
    }

    fn handle_menu_selection(&mut self) -> Task<Message> {
        match self.current_screen {
            AppScreen::AccountManagement => match self.menu_selection_index {
                0 => {
                    self.current_screen = AppScreen::ProviderSelection;
                    self.menu_selection_index = 0;
                    Task::done(Message::ShowProviderSelection)
                }
                1 => {
                    self.current_screen = AppScreen::Main;
                    self.menu_selection_index = 0;
                    Task::done(Message::BackToMain)
                }
                _ => Task::none(),
            },
            AppScreen::ProviderSelection => match self.menu_selection_index {
                0 => {
                    self.selected_provider = Some(AccountProvider::Gmail);
                    Task::done(Message::SelectProvider(AccountProvider::Gmail))
                }
                1 => {
                    self.selected_provider = Some(AccountProvider::Other);
                    Task::done(Message::SelectProvider(AccountProvider::Other))
                }
                2 => {
                    self.current_screen = AppScreen::AccountManagement;
                    self.menu_selection_index = 0;
                    Task::done(Message::ShowAccountManagement)
                }
                _ => Task::none(),
            },
            AppScreen::OAuthInProgress => {
                self.current_screen = AppScreen::AccountManagement;
                self.menu_selection_index = 0;
                Task::done(Message::ShowAccountManagement)
            }
            AppScreen::ManualAccountSetup => {
                self.current_screen = AppScreen::ProviderSelection;
                self.menu_selection_index = 0;
                Task::done(Message::ShowProviderSelection)
            }
            AppScreen::Main => Task::none(),
        }
    }

    fn handle_menu_go_back(&mut self) -> Task<Message> {
        match self.current_screen {
            AppScreen::AccountManagement => {
                self.current_screen = AppScreen::Main;
                self.menu_selection_index = 0;
                Task::done(Message::BackToMain)
            }
            AppScreen::ProviderSelection => {
                self.current_screen = AppScreen::AccountManagement;
                self.menu_selection_index = 0;
                Task::done(Message::ShowAccountManagement)
            }
            AppScreen::OAuthInProgress => {
                self.current_screen = AppScreen::AccountManagement;
                self.menu_selection_index = 0;
                Task::done(Message::ShowAccountManagement)
            }
            AppScreen::ManualAccountSetup => {
                self.current_screen = AppScreen::ProviderSelection;
                self.menu_selection_index = 0;
                Task::done(Message::ShowProviderSelection)
            }
            AppScreen::Main => Task::none(),
        }
    }
}
