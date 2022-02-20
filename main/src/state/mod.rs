mod browser;
mod browser_tab;

use crate::{app::AppRuntime, ui::UI};
use gtk::prelude::*;

use browser::Browser;
use browser_tab::BrowserTab;
use shared::primitive::Size;
use url::Url;

pub struct AppState {
    pub ui: UI,
    pub runtime: AppRuntime,
    pub browser: Browser,
    tabs: Vec<BrowserTab>,
    pub active_tab: usize,
    pub viewport: Size
}

impl AppState {
    pub fn new(ui: UI, runtime: AppRuntime) -> Self {
        Self {
            ui,
            runtime,
            browser: Browser::new(),
            tabs: Vec::new(),
            active_tab: 0,
            viewport: Size::new(1200., 600.)
        }
    }

    pub fn new_tab(&mut self, url: Url, active: bool) -> &BrowserTab {
        let tab = BrowserTab::new(url);
        self.tabs.push(tab);
        
        if active {
            self.set_active_tab(self.tabs.len() - 1);
        }

        self.tabs.last().unwrap()
    }

    pub fn active_tab(&self) -> &BrowserTab {
        self.tabs.get(self.active_tab).unwrap()
    }

    pub fn set_active_tab(&mut self, index: usize) {
        self.active_tab = index;
        self.active_tab().paint(self.viewport.clone());
    }

    pub fn quit(&self) {
        self.ui.window.close();
    }
}

