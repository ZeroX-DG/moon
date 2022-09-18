use shared::primitive::{Rect, Size};

pub struct LayoutContext {
    pub viewport: Rect,
    pub measure_text_fn: Box<dyn FnMut(&str, f32) -> Size>,
}

impl LayoutContext {
    pub fn measure_text(&mut self, content: &str, font_size: f32) -> Size {
        (self.measure_text_fn)(content, font_size)
    }
}
