use browser::Browser;
use browser_window::BrowserWindow;
use gtk::{Application, prelude::*};

mod browser;
mod browser_window;

pub fn start_main() {
    let app = Application::builder()
        .application_id("org.moon.MoonBrowser")
        .build();

    app.connect_activate(|app| {
        let browser_window = BrowserWindow::new(&app);
        let browser = Browser::new(browser_window);
        browser.initialize();
    });
    app.run();
}

