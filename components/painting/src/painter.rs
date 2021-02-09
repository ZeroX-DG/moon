use super::color::Color;
use super::rect::Rect;

pub trait Painter {
    fn fill_rect(&mut self, rect: &Rect, color: &Color);
    fn stroke_rect(&mut self, rect: &Rect, color: &Color);
}
