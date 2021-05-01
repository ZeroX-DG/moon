use super::frame::Frame;
use super::paint::{Painter, OutputBitmap};

pub struct Page {
    main_frame: Frame,
}

impl Page {
    pub fn new() -> Self {
        Self {
            main_frame: Frame::new()
        }
    }

    pub fn set_size(&mut self, size: (u32, u32)) {
        self.main_frame.set_size(size.0, size.1);
    }

    pub fn load_html(&mut self, html: String) {
        self.main_frame.load_html(html);
    }

    pub fn load_css(&mut self, css: String) {
        self.main_frame.load_css(css);
    }

    pub async fn paint(&self, painter: &mut Painter) -> Option<OutputBitmap> {
        self.main_frame.paint(painter).await
    }
}

