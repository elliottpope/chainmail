use crate::app::{ChainmailApp, Message};
use crate::vim::VimMode;
use iced::widget::{center, column, container, row, scrollable, stack, text, Column, Container, Row, Space};
use iced::{alignment, Border, Color, Element, Length, Padding};

pub fn main_view(app: &ChainmailApp) -> Element<Message> {
    let content = column![
        search_bar(app),
        main_layout(app),
        status_bar(app),
    ]
    .spacing(0);

    let base_view = container(content)
        .width(Length::Fill)
        .height(Length::Fill);

    // Show command modal overlay when in Command mode
    if app.vim_state().mode == VimMode::Command {
        stack![
            base_view,
            command_modal(app),
        ]
        .into()
    } else {
        base_view.into()
    }
}

fn search_bar(app: &ChainmailApp) -> Element<Message> {
    let search_text = if app.search_query().is_empty() {
        text("Press '/' to search, ':' for commands").size(14)
    } else {
        text(format!("Search: {}", app.search_query())).size(14)
    };

    container(search_text)
        .width(Length::Fill)
        .padding(8)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn main_layout(app: &ChainmailApp) -> Element<Message> {
    let has_selection = app.selected_message().is_some();

    if has_selection {
        row![
            account_selector(app),
            email_list(app),
            email_display(app),
        ]
        .spacing(0)
        .height(Length::Fill)
        .into()
    } else {
        row![
            account_selector(app),
            email_list(app),
        ]
        .spacing(0)
        .height(Length::Fill)
        .into()
    }
}

fn account_selector(_app: &ChainmailApp) -> Element<Message> {
    let accounts_column = column![
        text("Accounts").size(16),
        Space::with_height(10),
        text("All Inboxes").size(14),
    ]
    .padding(10)
    .spacing(5);

    let scrollable_accounts = scrollable(accounts_column);

    container(scrollable_accounts)
        .width(200)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn email_list(app: &ChainmailApp) -> Element<Message> {
    let messages = app.messages();
    let selected_index = app.selected_message();
    let visual_range = app.visual_selection_range();

    let mut items = Column::new().spacing(0);

    if messages.is_empty() {
        items = items.push(
            container(text("No messages").size(14))
                .padding(10)
                .width(Length::Fill),
        );
    } else {
        for (index, msg_with_mb) in messages.iter().enumerate() {
            let msg = &msg_with_mb.message;
            let is_selected = selected_index == Some(index);
            let is_in_visual_selection = visual_range
                .map(|(start, end)| index >= start && index <= end)
                .unwrap_or(false);

            let subject = msg.subject.as_deref().unwrap_or("(No Subject)");
            let from = msg.from_addr.as_deref().unwrap_or("Unknown");
            let date_str = msg
                .date
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let seen_indicator = if msg.seen { " " } else { "●" };
            let flagged_indicator = if msg.flagged { "⭐" } else { " " };

            let item_content = column![
                row![
                    text(seen_indicator).size(12),
                    text(flagged_indicator).size(12),
                    text(from).size(14),
                    Space::with_width(Length::Fill),
                    text(date_str).size(12),
                ]
                .spacing(5),
                text(subject).size(14),
                text(format!("{} - {}", msg_with_mb.account_name, msg_with_mb.mailbox_name))
                    .size(11)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .spacing(3)
            .padding(8);

            let background_color = if is_selected {
                Color::from_rgb(0.3, 0.4, 0.5)
            } else if is_in_visual_selection {
                Color::from_rgb(0.25, 0.35, 0.45)
            } else {
                Color::from_rgb(0.12, 0.12, 0.12)
            };

            let item = container(item_content)
                .width(Length::Fill)
                .style(move |theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(background_color)),
                    border: Border {
                        color: Color::from_rgb(0.25, 0.25, 0.25),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                });

            items = items.push(item);
        }
    }

    let scrollable_list = scrollable(items);

    let list_width = if selected_index.is_some() {
        Length::FillPortion(1)
    } else {
        Length::Fill
    };

    container(scrollable_list)
        .width(list_width)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.12))),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn email_display(app: &ChainmailApp) -> Element<Message> {
    let messages = app.messages();
    let selected_index = app.selected_message();

    let content = if let Some(index) = selected_index {
        if let Some(msg_with_mb) = messages.get(index) {
            let msg = &msg_with_mb.message;

            column![
                text(msg.subject.as_deref().unwrap_or("(No Subject)"))
                    .size(20)
                    .color(Color::WHITE),
                Space::with_height(10),
                text(format!("From: {}", msg.from_addr.as_deref().unwrap_or("Unknown")))
                    .size(14)
                    .color(Color::from_rgb(0.8, 0.8, 0.8)),
                text(format!("To: {}", msg.to_addr.as_deref().unwrap_or("Unknown")))
                    .size(14)
                    .color(Color::from_rgb(0.8, 0.8, 0.8)),
                text(format!(
                    "Date: {}",
                    msg.date
                        .map(|d| d.to_rfc2822())
                        .unwrap_or_else(|| "Unknown".to_string())
                ))
                .size(14)
                .color(Color::from_rgb(0.8, 0.8, 0.8)),
                text(format!("Account: {} ({})", msg_with_mb.account_name, msg_with_mb.mailbox_name))
                    .size(12)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
                Space::with_height(20),
                text(msg.body_preview.as_deref().unwrap_or("(No preview available)"))
                    .size(14)
                    .color(Color::from_rgb(0.9, 0.9, 0.9)),
            ]
            .spacing(5)
            .padding(20)
        } else {
            column![text("No message selected").size(16)]
                .padding(20)
        }
    } else {
        column![text("No message selected").size(16)]
            .padding(20)
    };

    let scrollable_content = scrollable(content);

    container(scrollable_content)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn status_bar(app: &ChainmailApp) -> Element<Message> {
    let vim_mode = app.vim_state().mode;
    let mode_text = vim_mode.to_string();

    let mode_color = match vim_mode {
        VimMode::Normal => Color::from_rgb(0.5, 0.7, 1.0),
        VimMode::Insert => Color::from_rgb(0.5, 1.0, 0.5),
        VimMode::Visual => Color::from_rgb(1.0, 0.7, 0.3),
        VimMode::Command => Color::from_rgb(1.0, 1.0, 0.5),
    };

    let status_content = container(text(mode_text).size(14).color(mode_color))
        .padding(Padding::from([4, 10]));

    container(status_content)
        .width(Length::Fill)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.08))),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn command_modal(app: &ChainmailApp) -> Element<Message> {
    let command_text = format!(": {}", app.vim_state().get_command());

    let modal_content = container(text(command_text).size(16))
        .padding(Padding::from([8, 16]))
        .width(600)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                color: Color::from_rgb(1.0, 1.0, 0.5),
                width: 2.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    center(modal_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
