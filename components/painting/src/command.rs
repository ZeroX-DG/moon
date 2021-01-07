use serde::{Deserialize, Serialize};
use super::rect::Rect;
use super::color::Color;

#[derive(Debug, Serialize, Deserialize)]
pub enum DisplayCommand {
    FillRect(Rect, Color),
    StrokeRect(Rect, Color),
}