use super::modes::{VimMode, VimState};
use iced::keyboard::{self, Key, Modifiers};

#[derive(Debug, Clone)]
pub enum VimAction {
    None,
    MoveUp(usize),
    MoveDown(usize),
    MoveLeft(usize),
    MoveRight(usize),
    MoveToTop,
    MoveToBottom,
    MovePageUp,
    MovePageDown,
    SelectCurrent,
    EnterInsertMode,
    EnterVisualMode,
    EnterCommandMode,
    EnterNormalMode,
    ExecuteCommand(String),
    DeleteSelected,
    YankSelected,
    ToggleFlagged,
    MarkAsRead,
    MarkAsUnread,
    OpenEmail,
}

pub struct VimKeyHandler;

impl VimKeyHandler {
    pub fn handle_key_press(
        vim_state: &mut VimState,
        key: &Key,
        modifiers: &Modifiers,
    ) -> VimAction {
        match vim_state.mode {
            VimMode::Normal => Self::handle_normal_mode(vim_state, key, modifiers),
            VimMode::Insert => Self::handle_insert_mode(vim_state, key, modifiers),
            VimMode::Visual => Self::handle_visual_mode(vim_state, key, modifiers),
            VimMode::Command => Self::handle_command_mode(vim_state, key, modifiers),
        }
    }

    fn handle_normal_mode(vim_state: &mut VimState, key: &Key, modifiers: &Modifiers) -> VimAction {
        match key {
            Key::Character(c) => {
                let ch = c.chars().next().unwrap_or('\0');
                match ch {
                    // Movement
                    'j' => {
                        let count = vim_state.get_count();
                        vim_state.reset_count();
                        VimAction::MoveDown(count)
                    }
                    'k' => {
                        let count = vim_state.get_count();
                        vim_state.reset_count();
                        VimAction::MoveUp(count)
                    }
                    'h' => {
                        let count = vim_state.get_count();
                        vim_state.reset_count();
                        VimAction::MoveLeft(count)
                    }
                    'l' => {
                        let count = vim_state.get_count();
                        vim_state.reset_count();
                        VimAction::MoveRight(count)
                    }
                    'g' => {
                        if vim_state.get_pending_operator() == Some('g') {
                            vim_state.clear_pending_operator();
                            vim_state.reset_count();
                            VimAction::MoveToTop
                        } else {
                            vim_state.set_pending_operator('g');
                            VimAction::None
                        }
                    }
                    'G' => {
                        vim_state.reset_count();
                        VimAction::MoveToBottom
                    }

                    // Page movement
                    'd' if modifiers.control() => VimAction::MovePageDown,
                    'u' if modifiers.control() => VimAction::MovePageUp,

                    // Mode switching
                    'i' => {
                        vim_state.enter_mode(VimMode::Insert);
                        VimAction::EnterInsertMode
                    }
                    'v' => {
                        vim_state.enter_mode(VimMode::Visual);
                        VimAction::EnterVisualMode
                    }
                    ':' | ';' if ch == ':' || (ch == ';' && modifiers.shift()) => {
                        vim_state.enter_mode(VimMode::Command);
                        VimAction::EnterCommandMode
                    }
                    '/' | '?' if ch == '/' || (ch == '?' && !modifiers.shift()) => {
                        vim_state.enter_mode(VimMode::Command);
                        vim_state.append_to_command_buffer('f');
                        vim_state.append_to_command_buffer('i');
                        vim_state.append_to_command_buffer('n');
                        vim_state.append_to_command_buffer('d');
                        vim_state.append_to_command_buffer(' ');
                        VimAction::EnterCommandMode
                    }

                    // Actions
                    '\n' | '\r' => VimAction::OpenEmail,
                    '*' | '8' if ch == '*' || (ch == '8' && modifiers.shift()) => VimAction::ToggleFlagged,
                    'r' => VimAction::MarkAsRead,
                    'u' => VimAction::MarkAsUnread,
                    'd' if vim_state.get_pending_operator() == Some('d') => {
                        vim_state.clear_pending_operator();
                        VimAction::DeleteSelected
                    }
                    'd' => {
                        vim_state.set_pending_operator('d');
                        VimAction::None
                    }
                    'y' if vim_state.get_pending_operator() == Some('y') => {
                        vim_state.clear_pending_operator();
                        VimAction::YankSelected
                    }
                    'y' => {
                        vim_state.set_pending_operator('y');
                        VimAction::None
                    }

                    // Count
                    '0'..='9' => {
                        if ch == '0' && vim_state.count.is_none() {
                            VimAction::None
                        } else {
                            vim_state.add_count_digit(ch.to_digit(10).unwrap());
                            VimAction::None
                        }
                    }

                    _ => VimAction::None,
                }
            }
            Key::Named(named) => match named {
                keyboard::key::Named::ArrowDown => VimAction::MoveDown(1),
                keyboard::key::Named::ArrowUp => VimAction::MoveUp(1),
                keyboard::key::Named::ArrowLeft => VimAction::MoveLeft(1),
                keyboard::key::Named::ArrowRight => VimAction::MoveRight(1),
                keyboard::key::Named::Escape => {
                    vim_state.reset_count();
                    vim_state.clear_pending_operator();
                    VimAction::None
                }
                keyboard::key::Named::Enter => VimAction::OpenEmail,
                _ => VimAction::None,
            },
            _ => VimAction::None,
        }
    }

    fn handle_insert_mode(vim_state: &mut VimState, key: &Key, _modifiers: &Modifiers) -> VimAction {
        match key {
            Key::Named(keyboard::key::Named::Escape) => {
                vim_state.enter_mode(VimMode::Normal);
                VimAction::EnterNormalMode
            }
            _ => VimAction::None,
        }
    }

    fn handle_visual_mode(vim_state: &mut VimState, key: &Key, modifiers: &Modifiers) -> VimAction {
        match key {
            Key::Character(c) => {
                let ch = c.chars().next().unwrap_or('\0');
                match ch {
                    // Movement (same as normal mode)
                    'j' => {
                        let count = vim_state.get_count();
                        vim_state.reset_count();
                        VimAction::MoveDown(count)
                    }
                    'k' => {
                        let count = vim_state.get_count();
                        vim_state.reset_count();
                        VimAction::MoveUp(count)
                    }
                    'g' => {
                        if vim_state.get_pending_operator() == Some('g') {
                            vim_state.clear_pending_operator();
                            VimAction::MoveToTop
                        } else {
                            vim_state.set_pending_operator('g');
                            VimAction::None
                        }
                    }
                    'G' => VimAction::MoveToBottom,

                    // Actions on selection
                    'd' => {
                        vim_state.enter_mode(VimMode::Normal);
                        VimAction::DeleteSelected
                    }
                    'y' => {
                        vim_state.enter_mode(VimMode::Normal);
                        VimAction::YankSelected
                    }
                    '*' | '8' if ch == '*' || (ch == '8' && modifiers.shift()) => VimAction::ToggleFlagged,

                    // Count
                    '0'..='9' => {
                        if ch == '0' && vim_state.count.is_none() {
                            VimAction::None
                        } else {
                            vim_state.add_count_digit(ch.to_digit(10).unwrap());
                            VimAction::None
                        }
                    }

                    _ => VimAction::None,
                }
            }
            Key::Named(named) => match named {
                keyboard::key::Named::Escape => {
                    vim_state.enter_mode(VimMode::Normal);
                    VimAction::EnterNormalMode
                }
                keyboard::key::Named::ArrowDown => VimAction::MoveDown(1),
                keyboard::key::Named::ArrowUp => VimAction::MoveUp(1),
                _ => VimAction::None,
            },
            _ => VimAction::None,
        }
    }

    fn handle_command_mode(vim_state: &mut VimState, key: &Key, _modifiers: &Modifiers) -> VimAction {
        match key {
            Key::Character(c) => {
                let ch = c.chars().next().unwrap_or('\0');
                vim_state.append_to_command_buffer(ch);
                VimAction::None
            }
            Key::Named(named) => match named {
                keyboard::key::Named::Escape => {
                    vim_state.enter_mode(VimMode::Normal);
                    VimAction::EnterNormalMode
                }
                keyboard::key::Named::Enter => {
                    let command = vim_state.get_command().to_string();
                    vim_state.enter_mode(VimMode::Normal);
                    VimAction::ExecuteCommand(command)
                }
                keyboard::key::Named::Backspace => {
                    vim_state.backspace_command_buffer();
                    if vim_state.get_command().is_empty() {
                        vim_state.enter_mode(VimMode::Normal);
                        VimAction::EnterNormalMode
                    } else {
                        VimAction::None
                    }
                }
                _ => VimAction::None,
            },
            _ => VimAction::None,
        }
    }
}
