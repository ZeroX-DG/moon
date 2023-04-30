use iced::{Application, Settings};

mod app;
mod fonts;
mod render_client;
mod state;

pub fn start_main() -> iced::Result {
    app::Moon::run(Settings::default())
}
