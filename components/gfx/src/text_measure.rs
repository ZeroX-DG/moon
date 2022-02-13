use glyph_brush::{ab_glyph::FontArc, Extra, FontId, GlyphCruncher, Section, Text};
use shared::primitive::Size;
use crate::fonts;

pub struct TextMeasure {
    brush: glyph_brush::GlyphBrush<()>,
}

impl TextMeasure {
    pub fn new() -> Self {
        let font = FontArc::try_from_slice(fonts::FALLBACK).expect("Unable to load default font");
        let brush = glyph_brush::GlyphBrushBuilder::using_font(font).build();
        Self { brush }
    }

    pub fn measure(&mut self, content: &str, font_size: f32) -> Size {
        let section = Section {
            text: vec![Text {
                text: content,
                scale: font_size.into(),
                font_id: FontId(0),
                extra: Extra::default(),
            }],
            bounds: (f32::MAX, f32::MAX),
            ..Default::default()
        };
        if let Some(rect) = self.brush.glyph_bounds(section) {
            Size::new(rect.width(), rect.height())
        } else {
            Size::new(0., 0.)
        }
    }
}
