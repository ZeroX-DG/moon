use super::color::Color;
use super::primitive::{RRect, Rect};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DrawCommand {
    FillRect(Rect, Color),
    FillRRect(RRect, Color),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DisplayCommand {
    Draw(DrawCommand),
    GroupDraw(Vec<DrawCommand>)
}
