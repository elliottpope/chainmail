pub mod account_management;
pub mod layout;

use crate::app::{AppScreen, ChainmailApp, Message};
use iced::Element;

pub fn view(app: &ChainmailApp) -> Element<Message> {
    match app.current_screen() {
        AppScreen::Main => layout::main_view(app),
        _ => account_management::account_management_view(app),
    }
}
