use shared::primitive::*;
use shared::color::Color;

pub trait Painter {
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn fill_rrect(&mut self, rect: RRect, color: Color);
}
