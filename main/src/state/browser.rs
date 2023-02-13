use std::path::Path;

use flume::{Receiver, Sender};
use shared::primitive::{Point, Size};
use url::{parser::URLParser, Url};

use super::browser_tab::{BrowserTab, TabEvent, TabHandler};

fn start_tab(tab: BrowserTab) -> TabHandler {
    let handler = tab.handler();
    std::thread::spawn(|| {
        tab.run().expect("Tab crashed");
    });

    handler
}

type BrowserAction = Box<dyn FnOnce(&mut Browser) + Send>;

#[derive(Clone)]
pub struct BrowserHandler {
    action_tx: Sender<BrowserAction>,
    tab_event_rx: Receiver<(usize, TabEvent)>,
}

impl BrowserHandler {
    pub fn resize(&self, size: Size) {
        self.update(|browser| {
            let active_tab = browser.get_active_tab();
            active_tab.resize(size).unwrap();
        });
    }

    pub fn scroll(&self, y: f32) {
        self.update(move |browser| {
            let active_tab = browser.get_active_tab();
            active_tab.scroll(y).unwrap();
        });
    }

    pub fn handle_mouse_move(&self, mouse_coord: Point) {
        self.update(move |browser| {
            let active_tab = browser.get_active_tab();
            active_tab.handle_mouse_move(mouse_coord).unwrap();
        });
    }

    pub fn view_source_current_tab(&self) {
        self.update(|browser| {
            let active_tab = browser.get_active_tab();
            let active_tab_url = active_tab.info().url.lock().unwrap().as_str();

            if active_tab_url.starts_with("view-source:") {
                return;
            }

            let url = format!("view-source:{}", active_tab_url);
            active_tab.goto(url).unwrap();
        });
    }

    pub fn goto(&self, raw_url: String) {
        if raw_url.is_empty() {
            return;
        }

        self.update(move |browser| {
            let active_tab = browser.get_active_tab();
            active_tab.goto(raw_url).unwrap();
        });
    }

    pub fn reload(&self) {
        self.update(move |browser| {
            let active_tab = browser.get_active_tab();
            active_tab.reload().expect("Unable to reload");
        });
    }

    pub fn events(&self) -> Receiver<(usize, TabEvent)> {
        self.tab_event_rx.clone()
    }

    fn update(&self, action: impl FnOnce(&mut Browser) + Send + 'static) {
        self.action_tx.send(Box::new(action)).unwrap();
    }
}

pub struct Browser {
    home_url: Url,
    tab_handlers: Vec<TabHandler>,
    active_tab_index: usize,
    update_channel: (Sender<BrowserAction>, Receiver<BrowserAction>),
    tab_event_channel: (Sender<(usize, TabEvent)>, Receiver<(usize, TabEvent)>)
}

impl Browser {
    pub fn new() -> Self {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_str()
            .unwrap();

        let home_url = URLParser::parse(
            &format!("file://{}/fixtures/test.html", workspace_root),
            None,
        )
        .unwrap();

        let initial_tab = BrowserTab::new(home_url.clone());
        let initial_tab_handler = start_tab(initial_tab);

        Self {
            home_url,
            tab_handlers: vec![initial_tab_handler],
            active_tab_index: 0,
            update_channel: flume::unbounded(),
            tab_event_channel: flume::unbounded(),
        }
    }

    pub fn handler(&self) -> BrowserHandler {
        let (tx, _) = &self.update_channel;
        let (_, rx) = &self.tab_event_channel;
        BrowserHandler {
            action_tx: tx.clone(),
            tab_event_rx: rx.clone()
        }
    }

    pub fn get_active_tab(&self) -> &TabHandler {
        self.tab_handlers
            .get(self.active_tab_index)
            .as_ref()
            .unwrap()
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let active_tab = self.get_active_tab();
        active_tab.goto(self.home_url.to_string()).unwrap();

        enum Event {
            UpdateEvent(BrowserAction),
            TabEvent((usize, TabEvent)),
        }

        loop {
            let mut selector = flume::Selector::new();

            for (tab_index, tab) in self.tab_handlers.iter().enumerate() {
                selector = selector.recv(tab.events(), move |event| {
                    event.map(|e| Event::TabEvent((tab_index, e)))
                });
            }

            let (tab_event_tx, _) = &self.tab_event_channel;
            let (_, update_receiver) = &self.update_channel;
            selector = selector.recv(update_receiver, |event| {
                event.map(|e| Event::UpdateEvent(e))
            });

            let event = selector.wait()?;

            match event {
                Event::TabEvent((tab_index, event)) => tab_event_tx.send((tab_index, event))?, 
                Event::UpdateEvent(action) => action(&mut self),
            }
        }
    }
}
