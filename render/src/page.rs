use super::frame::Frame;
use super::paint::{Painter, OutputBitmap};

pub struct Page {
    main_frame: Frame,
    last_output_bitmap: Option<OutputBitmap>
}

impl Page {
    pub fn new() -> Self {
        Self {
            main_frame: Frame::new(),
            last_output_bitmap: None
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

    pub fn last_output_bitmap(&self) -> Option<OutputBitmap> {
        self.last_output_bitmap.clone()
    }

    pub async fn paint(&mut self, painter: &mut Painter) -> Option<OutputBitmap> {
        let bitmap = self.main_frame.paint(painter).await;
        self.last_output_bitmap = bitmap.clone();
        bitmap
    }
}

