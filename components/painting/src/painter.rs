use super::rect::Rect;
use super::color::Color;

pub trait Painter {
    fn fill_rect(&mut self, rect: &Rect, color: &Color);
    fn stroke_rect(&mut self, rect: &Rect, color: &Color);
}