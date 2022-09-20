use std::sync::{Arc, Mutex};

use crate::render_client::RenderClient;
use flume::{Receiver, Selector, Sender};
use render::OutputEvent;
use shared::primitive::{Point, Size};
use url::Url;

pub enum TabAction {
    Resize(Size),
    Scroll(f32),
    MouseMove(Point),
    Goto(String),
    Reload,
}

pub enum TabEvent {
    URLChanged(Url),
    FrameReceived(Vec<u8>),
    TitleChanged(String),
    LoadingStart,
    LoadingFinished,
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

    pub fn scroll(&self, y: f32) -> anyhow::Result<()> {
        self.sender.send(TabAction::Scroll(y))?;
        Ok(())
    }

    pub fn goto(&self, url: String) -> anyhow::Result<()> {
        self.sender.send(TabAction::Goto(url))?;
        Ok(())
    }

    pub fn reload(&self) -> anyhow::Result<()> {
        self.sender.send(TabAction::Reload)?;
        Ok(())
    }

    pub fn events(&self) -> &Receiver<TabEvent> {
        &self.receiver
    }

    pub fn info(&self) -> Arc<TabInfo> {
        self.info.clone()
    }

    pub fn handle_mouse_move(&self, mouse_coord: shared::primitive::Point) -> anyhow::Result<()> {
        self.sender.send(TabAction::MouseMove(mouse_coord))?;
        Ok(())
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
            TabAction::Scroll(y) => self.client.scroll(y),
            TabAction::MouseMove(coord) => self.client.mouse_move(coord),
            TabAction::Goto(url) => self.goto(url)?,
            TabAction::Reload => self.reload()?,
        }
        Ok(())
    }

    fn handle_render_engine_event(&self, event: OutputEvent) -> anyhow::Result<()> {
        match event {
            OutputEvent::FrameRendered(frame) => self.emit_event(TabEvent::FrameReceived(frame))?,
            OutputEvent::TitleChanged(title) => self.emit_event(TabEvent::TitleChanged(title))?,
            OutputEvent::URLChanged(url) => self.emit_event(TabEvent::URLChanged(url))?,
            OutputEvent::LoadingStarted => self.emit_event(TabEvent::LoadingStart)?,
            OutputEvent::LoadingFinished => self.emit_event(TabEvent::LoadingFinished)?,
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
    fn reload(&self) -> anyhow::Result<()> {
        self.client.reload();
        Ok(())
    }

    fn goto(&self, url: String) -> anyhow::Result<()> {
        self.client.load_raw_url(url);
        Ok(())
    }
}
