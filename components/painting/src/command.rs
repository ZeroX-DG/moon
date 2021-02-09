use super::color::Color;
use super::rect::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DisplayCommand {
    FillRect(Rect, Color),
    StrokeRect(Rect, Color),
}
