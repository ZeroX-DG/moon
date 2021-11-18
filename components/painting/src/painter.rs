use shared::color::Color;
use shared::primitive::*;

pub trait Painter {
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn fill_rrect(&mut self, rect: RRect, color: Color);
    fn fill_text(&mut self, content: String, bounds: Rect, color: Color, size: f32);
}
