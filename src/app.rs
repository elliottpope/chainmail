use crate::db::{models::MessageWithMailbox, Database};
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
}

impl ChainmailApp {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let db_path = "sqlite://chainmail.db";
        let db = Database::new(db_path).await?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            vim_state: VimState::new(),
            selected_account: None,
            selected_message: Some(0),
            messages: Vec::new(),
            displayed_messages: Vec::new(),
            search_query: String::new(),
            visual_selection_start: None,
        })
    }

    fn handle_vim_action(&mut self, action: VimAction) -> Task<Message> {
        match action {
            VimAction::MoveDown(count) => {
                if let Some(current) = self.selected_message {
                    let new_index = (current + count).min(self.displayed_messages.len().saturating_sub(1));
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

    pub async fn load_all_messages(db: Arc<Mutex<Database>>) -> Result<Vec<MessageWithMailbox>, anyhow::Error> {
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
}
