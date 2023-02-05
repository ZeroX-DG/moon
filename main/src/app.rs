use iced::{Application, executor, Theme, Command, Renderer, widget::{container, column}};

use crate::ui::primary_bar::PrimaryBar;

pub struct Moon {
    primary_bar: PrimaryBar,
}

#[derive(Debug, Clone)]
pub enum Message {
    URLInputContentChanged(String),
}

impl Application for Moon {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        let instance = Moon {
            primary_bar: PrimaryBar::new()
        };
        (instance, Command::none())
    }

    fn title(&self) -> String {
        String::from("Moon browser")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.primary_bar.update(message);
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Renderer<Self::Theme>> {
        let content = column![
            self.primary_bar.view()
        ];
        container(content).into()
    }
}
