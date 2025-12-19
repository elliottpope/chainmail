mod app;
mod db;
mod imap;
mod ui;
mod vim;

use app::{ChainmailApp, Message};
use iced::{window, Size, Task};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(
        "Chainmail - Vim Email Client",
        ChainmailApp::update,
        ChainmailApp::view,
    )
    .theme(ChainmailApp::theme)
    .subscription(ChainmailApp::subscription)
    .window_size(Size::new(1400.0, 900.0))
    .run_with(|| {
        let app_future = async {
            match ChainmailApp::new().await {
                Ok(app) => app,
                Err(e) => {
                    eprintln!("Failed to initialize app: {}", e);
                    std::process::exit(1);
                }
            }
        };

        let app = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(app_future);

        let db = app.db.clone();
        let load_task = Task::perform(
            async move {
                ChainmailApp::load_all_messages(db)
                    .await
                    .unwrap_or_default()
            },
            Message::LoadMessages,
        );

        (app, load_task)
    })
}
