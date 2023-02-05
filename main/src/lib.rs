use iced::{Application, Settings};

mod app;
mod render_client;
mod ui;

pub fn start_main() -> iced::Result {
    app::Moon::run(Settings::default())
}
