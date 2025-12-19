use iced::keyboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl VimMode {
    pub fn to_string(&self) -> &'static str {
        match self {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Visual => "VISUAL",
            VimMode::Command => "COMMAND",
        }
    }
}

#[derive(Debug, Clone)]
pub struct VimState {
    pub mode: VimMode,
    pub command_buffer: String,
    pub count: Option<usize>,
    pub pending_operator: Option<char>,
}

impl Default for VimState {
    fn default() -> Self {
        Self {
            mode: VimMode::Normal,
            command_buffer: String::new(),
            count: None,
            pending_operator: None,
        }
    }
}

impl VimState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enter_mode(&mut self, mode: VimMode) {
        self.mode = mode;
        self.command_buffer.clear();
        self.count = None;
        self.pending_operator = None;
    }

    pub fn append_to_command_buffer(&mut self, c: char) {
        if self.mode == VimMode::Command {
            self.command_buffer.push(c);
        }
    }

    pub fn backspace_command_buffer(&mut self) {
        if self.mode == VimMode::Command {
            self.command_buffer.pop();
        }
    }

    pub fn get_command(&self) -> &str {
        &self.command_buffer
    }

    pub fn clear_command(&mut self) {
        self.command_buffer.clear();
    }

    pub fn add_count_digit(&mut self, digit: u32) {
        if self.mode == VimMode::Normal || self.mode == VimMode::Visual {
            let current = self.count.unwrap_or(0);
            self.count = Some(current * 10 + digit as usize);
        }
    }

    pub fn get_count(&self) -> usize {
        self.count.unwrap_or(1)
    }

    pub fn reset_count(&mut self) {
        self.count = None;
    }

    pub fn set_pending_operator(&mut self, op: char) {
        self.pending_operator = Some(op);
    }

    pub fn get_pending_operator(&self) -> Option<char> {
        self.pending_operator
    }

    pub fn clear_pending_operator(&mut self) {
        self.pending_operator = None;
    }
}
