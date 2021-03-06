use super::color::Color;
use super::primitive::{RRect, Rect};

pub trait Painter {
    fn fill_rect(&mut self, rect: &Rect, color: &Color);
    fn fill_rrect(&mut self, rect: &RRect, color: &Color);
    fn stroke_rect(&mut self, rect: &Rect, color: &Color);
}
