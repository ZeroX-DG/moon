use std::sync::{Arc, Mutex};

use crate::render_client::RenderClient;
use flume::{Receiver, Selector, Sender};
use render::OutputEvent;
use shared::byte_string::ByteString;
use shared::primitive::Size;
use url::Url;

pub enum TabAction {
    Resize(Size),
    Goto(Url),
    ShowError { title: String, body: String },
}

pub enum TabEvent {
    URLChanged(Url),
    FrameReceived(Vec<u8>),
    TitleChanged(String),
}

pub struct TabHandler {
    sender: Sender<TabAction>,
    receiver: Receiver<TabEvent>,
    info: Arc<TabInfo>,
}

impl TabHandler {
    pub fn resize(&self, size: Size) -> anyhow::Result<()> {
        self.sender.send(TabAction::Resize(size))?;
        Ok(())
    }

    pub fn goto(&self, url: Url) -> anyhow::Result<()> {
        self.sender.send(TabAction::Goto(url))?;
        Ok(())
    }

    pub fn show_error(&self, title: String, body: String) -> anyhow::Result<()> {
        self.sender.send(TabAction::ShowError { title, body })?;
        Ok(())
    }

    pub fn events(&self) -> &Receiver<TabEvent> {
        &self.receiver
    }

    pub fn info(&self) -> Arc<TabInfo> {
        self.info.clone()
    }
}

pub struct TabInfo {
    pub url: Mutex<Url>,
}

pub struct BrowserTab {
    info: Arc<TabInfo>,
    client: RenderClient,
    action_channel: (Sender<TabAction>, Receiver<TabAction>),
    event_channel: (Sender<TabEvent>, Receiver<TabEvent>),
}

impl BrowserTab {
    pub fn new(url: Url) -> Self {
        let client = RenderClient::new();
        client.wait_till_ready();

        let info = TabInfo {
            url: Mutex::new(url),
        };

        Self {
            info: Arc::new(info),
            client,
            action_channel: flume::unbounded(),
            event_channel: flume::unbounded(),
        }
    }

    pub fn run(self) -> anyhow::Result<()> {
        let (_, tab_action_rx) = &self.action_channel;
        let render_engine_events = self.client.events();

        enum Event {
            TabAction(TabAction),
            RenderEngineEvent(OutputEvent),
        }

        loop {
            let event = Selector::new()
                .recv(&tab_action_rx, |event| event.map(|e| Event::TabAction(e)))
                .recv(&render_engine_events, |event| {
                    event.map(|e| Event::RenderEngineEvent(e))
                })
                .wait()?;

            match event {
                Event::TabAction(event) => self.handle_tab_action(event)?,
                Event::RenderEngineEvent(event) => self.handle_render_engine_event(event)?,
            }
        }
    }

    pub fn handler(&self) -> TabHandler {
        let (sender, receiver) = self.channel();
        let info = self.info.clone();
        TabHandler {
            sender,
            receiver,
            info,
        }
    }

    fn channel(&self) -> (Sender<TabAction>, Receiver<TabEvent>) {
        let (_, receiver) = &self.event_channel;
        let (sender, _) = &self.action_channel;
        (sender.clone(), receiver.clone())
    }

    fn handle_tab_action(&self, event: TabAction) -> anyhow::Result<()> {
        match event {
            TabAction::Resize(new_size) => self.client.resize(new_size),
            TabAction::Goto(url) => self.goto(url)?,
            TabAction::ShowError { title, body } => self.load_error(&title, &body),
        }
        Ok(())
    }

    fn handle_render_engine_event(&self, event: OutputEvent) -> anyhow::Result<()> {
        match event {
            OutputEvent::FrameRendered(frame) => self.emit_event(TabEvent::FrameReceived(frame))?,
            OutputEvent::TitleChanged(title) => self.emit_event(TabEvent::TitleChanged(title))?,
        }

        Ok(())
    }

    fn emit_event(&self, event: TabEvent) -> anyhow::Result<()> {
        let (sender, _) = &self.event_channel;
        sender.send(event)?;
        Ok(())
    }
}

impl BrowserTab {
    fn goto(&self, url: Url) -> anyhow::Result<()> {
        *self.info.url.lock().unwrap() = url.clone();
        self.load(&url)?;
        self.change_url(url)?;
        Ok(())
    }

    fn load(&self, url: &Url) -> anyhow::Result<()> {
        match url.scheme.as_str() {
            "http" | "https" | "file" => self.load_html(),
            "view-source" => self.load_source(),
            _ => self.load_not_supported(),
        }
        Ok(())
    }

    fn change_url(&self, url: Url) -> anyhow::Result<()> {
        self.emit_event(TabEvent::URLChanged(url))?;
        Ok(())
    }

    fn load_error(&self, title: &str, error: &str) {
        let current_url = self.info.url.lock().unwrap().clone();
        let source_html = self.get_error_page_content(title, error);
        self.client.load_html(source_html, current_url);
    }

    fn get_error_page_content(&self, title: &str, error: &str) -> String {
        format!(
            "
            <html>
                <style>
                    body {{ background-color: #262ded }}
                    #error-content {{
                        width: 500px;
                        margin: 0 auto;
                        margin-top: 50px;
                        color: white;
                    }}
                </style>
                <div id='error-content'>
                    <h1>{}</h1>
                    <p>{}</p>
                </div>
            </html>
        ",
            title, error
        )
    }

    fn load_html(&self) {
        // let current_url = self.info.url.lock().unwrap().clone();
        // match ResourceLoader::global().load(&current_url) {
        //     Ok(bytes) => {
        //         let html = ByteString::new(&bytes);
        //         self.client.load_html(html.to_string(), current_url);
        //     }
        //     Err(e) => {
        //         self.load_error("Aw, Snap!", &e.get_friendly_message());
        //     }
        // }
    }

    fn load_source(&self) {
        // let current_url = self.info.url.lock().unwrap().clone();
        // match ResourceLoader::global().load(&current_url) {
        //     Ok(bytes) => {
        //         let raw_html_string = ByteString::new(&bytes).to_string();
        //         let raw_html = html_escape::encode_text(&raw_html_string);
        //         let source_html = format!("<html><pre>{}</pre></html>", raw_html);

        //         self.client.load_html(source_html, current_url);
        //     }
        //     Err(e) => {
        //         self.load_error("Aw, Snap!", &e.get_friendly_message());
        //     }
        // }
    }

    fn load_not_supported(&self) {
        let current_url = self.info.url.lock().unwrap().clone();
        let error = format!(
            "Unable to load resource via unsupported protocol: {}",
            current_url.scheme
        );
        self.load_error("Unsupported Protocol", &error);
    }
}
