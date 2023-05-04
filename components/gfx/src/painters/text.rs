use shared::{color::Color, primitive::Rect};

use crate::text::Text;

pub struct TextPainter {
    texts: Vec<Text>,
}

impl TextPainter {
    pub fn new() -> Self {
        Self { texts: Vec::new() }
    }

    pub fn fill_text(
        &mut self,
        content: String,
        bounds: Rect,
        color: Color,
        size: f32,
        bold: bool,
    ) {
        self.texts.push(Text {
            content,
            bounds,
            color,
            size,
            bold,
        })
    }

    pub fn texts(&self) -> &[Text] {
        &self.texts
    }

    pub fn clear(&mut self) {
        self.texts.clear();
    }
}
