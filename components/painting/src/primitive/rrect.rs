use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: Radius
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Radius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32
}

impl RRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32, r: Radius) -> Self {
        Self {
            x,
            y,
            width: w,
            height: h,
            radius: r
        }
    }
}

impl Radius {
    pub fn new(tl: f32, tr: f32, bl: f32, br: f32) -> Self {
        Self {
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br
        }
    }
}