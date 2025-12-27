use crate::app::{AppScreen, ChainmailApp, Message};
use crate::oauth::AccountProvider;
use iced::widget::{button, column, container, row, text, text_input, Column, Space};
use iced::{alignment, Border, Color, Element, Length, Padding};

pub fn account_management_view(app: &ChainmailApp) -> Element<Message> {
    match app.current_screen() {
        AppScreen::AccountManagement => account_management_menu(),
        AppScreen::ProviderSelection => provider_selection_view(),
        AppScreen::OAuthInProgress => oauth_in_progress_view(),
        AppScreen::ManualAccountSetup => manual_account_setup_view(app),
        _ => {
            container(text("Invalid screen"))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        }
    }
}

fn account_management_menu() -> Element<'static, Message> {
    let content = column![
        text("Account Management")
            .size(32)
            .color(Color::WHITE),
        Space::with_height(40),
        text("Add a new email account to Chainmail")
            .size(16)
            .color(Color::from_rgb(0.8, 0.8, 0.8)),
        Space::with_height(30),
        button(
            container(text("Add Account"))
                .padding(15)
                .width(Length::Fill)
                .center_x(Length::Fill)
        )
        .on_press(Message::ShowProviderSelection)
        .width(300),
        Space::with_height(15),
        button(
            container(text("Back to Mail"))
                .padding(15)
                .width(Length::Fill)
                .center_x(Length::Fill)
        )
        .on_press(Message::BackToMain)
        .width(300),
        Space::with_height(20),
        text("Tip: Use :account or :acc to access this menu")
            .size(12)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
    ]
    .spacing(0)
    .padding(40)
    .align_x(alignment::Horizontal::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
}

fn provider_selection_view() -> Element<'static, Message> {
    let content = column![
        text("Select Email Provider")
            .size(32)
            .color(Color::WHITE),
        Space::with_height(40),
        text("Choose your email provider:")
            .size(16)
            .color(Color::from_rgb(0.8, 0.8, 0.8)),
        Space::with_height(30),
        button(
            container(
                column![
                    text("Gmail")
                        .size(20)
                        .color(Color::WHITE),
                    Space::with_height(5),
                    text("Secure OAuth 2.0 authentication")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .align_x(alignment::Horizontal::Center)
            )
            .padding(20)
            .width(Length::Fill)
            .center_x(Length::Fill)
        )
        .on_press(Message::SelectProvider(AccountProvider::Gmail))
        .width(400),
        Space::with_height(15),
        button(
            container(
                column![
                    text("Other Provider")
                        .size(20)
                        .color(Color::WHITE),
                    Space::with_height(5),
                    text("Manual IMAP configuration (coming soon)")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .align_x(alignment::Horizontal::Center)
            )
            .padding(20)
            .width(Length::Fill)
            .center_x(Length::Fill)
        )
        .width(400),
        Space::with_height(30),
        button(
            container(text("Back"))
                .padding(10)
                .width(Length::Fill)
                .center_x(Length::Fill)
        )
        .on_press(Message::ShowAccountManagement)
        .width(200),
    ]
    .spacing(0)
    .padding(40)
    .align_x(alignment::Horizontal::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
}

fn oauth_in_progress_view() -> Element<'static, Message> {
    let content = column![
        text("🔐 Authorization in Progress")
            .size(32)
            .color(Color::WHITE),
        Space::with_height(40),
        text("Your browser should have opened to authorize Chainmail")
            .size(16)
            .color(Color::from_rgb(0.8, 0.8, 0.8)),
        Space::with_height(20),
        text("Steps:")
            .size(14)
            .color(Color::from_rgb(0.9, 0.9, 0.9)),
        Space::with_height(10),
        text("1. Sign in to your Gmail account")
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
        text("2. Review the permissions")
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
        text("3. Click 'Allow' to grant access")
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
        text("4. You'll be redirected back automatically")
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
        Space::with_height(30),
        text("Waiting for authorization...")
            .size(14)
            .color(Color::from_rgb(0.5, 0.7, 1.0)),
        Space::with_height(30),
        button(
            container(text("Cancel"))
                .padding(10)
                .width(Length::Fill)
                .center_x(Length::Fill)
        )
        .on_press(Message::ShowAccountManagement)
        .width(200),
    ]
    .spacing(5)
    .padding(40)
    .align_x(alignment::Horizontal::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
}

fn manual_account_setup_view(app: &ChainmailApp) -> Element<Message> {
    let content = column![
        text("Manual Account Setup")
            .size(32)
            .color(Color::WHITE),
        Space::with_height(40),
        text("This feature is coming soon!")
            .size(16)
            .color(Color::from_rgb(0.8, 0.8, 0.8)),
        Space::with_height(20),
        text("For now, please use Gmail with OAuth authentication.")
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
        Space::with_height(30),
        button(
            container(text("Back"))
                .padding(10)
                .width(Length::Fill)
                .center_x(Length::Fill)
        )
        .on_press(Message::ShowProviderSelection)
        .width(200),
    ]
    .spacing(0)
    .padding(40)
    .align_x(alignment::Horizontal::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
}
