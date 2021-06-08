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

    pub fn resize(&mut self, size: (u32, u32)) {
        self.main_frame.resize(size);
    }

    pub fn load_html(&mut self, html: String) {
        self.main_frame.load_html(html);
    }
}
