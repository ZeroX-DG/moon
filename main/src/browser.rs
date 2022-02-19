use crate::browser_window::BrowserWindow;

pub struct Browser {
    window: BrowserWindow
}

impl Browser {
    pub fn new(window: BrowserWindow) -> Self {
        Self {
            window
        }
    }

    pub fn initialize(&self) {
        self.window.initialize();
    }
}

