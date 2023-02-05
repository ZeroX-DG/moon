use iced::widget::{row, text_input};

use crate::app::Message;

pub struct PrimaryBar {
    url_input_box: URLInputBox,
}

impl PrimaryBar {
    pub fn new() -> Self {
        Self {
            url_input_box: URLInputBox::new()
        }
    }

    pub fn update(&mut self, message: Message) {
        self.url_input_box.update(message);
    }

    pub fn view(&self) -> iced::Element<Message> {
        row![
            self.url_input_box.view()
        ].into()
    }
}

pub struct URLInputBox {
    content: String
}

impl URLInputBox {
    pub fn new() -> Self {
        Self {
            content: String::new()
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::URLInputContentChanged(content) => {
                self.content = content;
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        text_input("Go to...", &self.content, Message::URLInputContentChanged).into()
    }
}
