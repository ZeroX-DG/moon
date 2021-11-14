use serde::{Deserialize, Serialize};
use shared::color::Color;
use shared::primitive::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum DrawCommand {
    FillRect(Rect, Color),
    FillRRect(RRect, Color),
    FillText(String, Rect, Color, f32),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DisplayCommand {
    Draw(DrawCommand),
    GroupDraw(Vec<DrawCommand>),
}
