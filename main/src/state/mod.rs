mod browser;
mod browser_tab;

use crate::{app::AppRuntime, ui::UI};

use browser::Browser;

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
            handler,
        }
    }

    pub fn browser(&self) -> &BrowserHandler {
        &self.handler
    }
}
