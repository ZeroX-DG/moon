use iced::{Application, executor, Theme, Command, Renderer, widget::{container, column, text_input, image}};
use crate::browser::Browser;

pub struct Moon {
    browser: Browser,
    url_input_content: String,
    content_width: u32,
    content_height: u32,
    content_data: Vec<u8>
}

#[derive(Debug, Clone)]
pub enum Message {
    URLInputContentChanged(String),
    URLNavigationTriggered
}

impl Application for Moon {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut browser = Browser::new();
        browser.new_tab();
        let instance = Moon {
            browser,
            url_input_content: String::new(),
            content_width: 0,
            content_height: 0,
            content_data: Vec::new()
        };
        (instance, Command::none())
    }

    fn title(&self) -> String {
        String::from("Moon browser")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::URLNavigationTriggered => {
                let url = self.url_input_content.clone();
                self.browser.current_tab_mut().unwrap().goto_raw_url(url);
            }
            Message::URLInputContentChanged(url) => {
                self.url_input_content = url;
            }
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message, Renderer<Self::Theme>> {
        let content = column![
            primary_bar(&self.url_input_content),
            content_area(self.content_width, self.content_height, self.content_data.clone()),
        ];
        container(content).into()
    }
}

fn primary_bar(
    url_content: &str
) -> iced::Element<'static, Message> {
    text_input("Go to...", url_content, Message::URLInputContentChanged)
        .on_submit(Message::URLNavigationTriggered)
        .into()
}

fn content_area(
    width: u32,
    height: u32,
    content: Vec<u8>
) -> iced::Element<'static, Message> {
    let image_handle = image::Handle::from_pixels(width, height, content);
        iced::widget::image(image_handle)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
}
