mod browser;
mod browser_tab;

use crate::{app::AppRuntime, ui::UI};

use browser::Browser;
use gtk::{
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::Bytes,
};
use url::Url;

use self::browser::BrowserHandler;

pub struct AppState {
    pub ui: UI,
    pub runtime: AppRuntime,
    handler: BrowserHandler,
}

impl AppState {
    pub fn new(ui: UI, runtime: AppRuntime) -> Self {
        let browser = Browser::new();
        let handler = browser.handler();
        let _ = std::thread::spawn(move || {
            browser.run().expect("Browser crashed");
        });

        Self {
            ui,
            runtime,
            handler
        }
    }

    pub fn browser(&self) -> &BrowserHandler {
        &self.handler
    }

    pub fn update_url(&mut self, url: Url) {
        self.ui.set_url(&url.as_str());
    }

    pub fn update_web_content(&mut self, bitmap: Vec<u8>) {
        let (width, height) = self.ui.content_area.render_area_size();

        if (width * height * 4) as usize > bitmap.len() {
            return;
        }

        let bytes = Bytes::from_owned(bitmap);
        let pixbuf = Pixbuf::from_bytes(&bytes, Colorspace::Rgb, true, 8, width, height, width * 4);
        self.ui.set_content_pixbuf(pixbuf);
    }
}
