use shared::primitive::Size;
use url::Url;

use super::frame::Frame;

pub struct Page {
    main_frame: Frame,
}

impl Page {
    pub fn new() -> Self {
        Self {
            main_frame: Frame::new(),
        }
    }

    pub fn main_frame(&self) -> &Frame {
        &self.main_frame
    }

    pub fn resize(&mut self, size: Size) {
        self.main_frame.resize(size);
    }

    pub fn load_html(&mut self, html: String, base_url: Url) {
        self.main_frame.load_html(html, base_url);
    }
}
