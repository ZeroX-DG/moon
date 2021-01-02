use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Paint {
    pub style: PaintStyle,
    pub color: PaintColor,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PaintStyle {
    Fill,
    Stroke,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaintColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for PaintColor {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}
