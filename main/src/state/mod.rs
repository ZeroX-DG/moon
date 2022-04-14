mod browser;
mod browser_tab;

use crate::{app::AppRuntime, ui::UI};

use browser::Browser;
use browser_tab::BrowserTab;
use gtk::{
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::Bytes,
    traits::WidgetExt,
};
use shared::primitive::Size;
use url::Url;

pub struct AppState {
    pub ui: UI,
    pub runtime: AppRuntime,
    pub browser: Browser,
    tabs: Vec<BrowserTab>,
    pub active_tab: usize,
}

impl AppState {
    pub fn new(ui: UI, runtime: AppRuntime) -> Self {
        Self {
            ui,
            runtime,
            browser: Browser::new(),
            tabs: Vec::new(),
            active_tab: 0,
        }
    }

    pub fn new_tab(&mut self, url: Url, active: bool) -> &BrowserTab {
        let tab = BrowserTab::new(url.clone());
        let viewport = Size::new(
            self.ui.content_area.allocated_width() as f32,
            self.ui.content_area.allocated_height() as f32,
        );
        tab.resize(viewport);
        tab.load();
        self.tabs.push(tab);

        if active {
            self.set_active_tab(self.tabs.len() - 1);
        }

        self.tabs.last().unwrap()
    }

    pub fn active_tab_mut(&mut self) -> &mut BrowserTab {
        self.tabs.get_mut(self.active_tab).unwrap()
    }

    pub fn set_active_tab(&mut self, index: usize) {
        // set current active tab to not active
        self.active_tab_mut().set_active(false);

        // set new active tab to active & repaint
        self.active_tab = index;
        self.active_tab_mut().set_active(true);
    }

    pub fn on_active_tab_bitmap(&mut self, bitmap: Vec<u8>) {
        let bytes = Bytes::from_owned(bitmap);
        let pixbuf = Pixbuf::from_bytes(
            &bytes,
            Colorspace::Rgb,
            true,
            8,
            self.ui.content_area.allocated_width(),
            self.ui.content_area.allocated_height(),
            self.ui.content_area.allocated_width() * 4,
        );
        self.ui.set_content_pixbuf(pixbuf);
    }
}
