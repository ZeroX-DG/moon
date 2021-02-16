use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub corners: Corners
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Corners {
    pub top_left: Radii,
    pub top_right: Radii,
    pub bottom_left: Radii,
    pub bottom_right: Radii
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Radii(f32, f32);

impl RRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32, corners: Corners) -> Self {
        Self {
            x,
            y,
            width: w,
            height: h,
            corners
        }
    }
}

impl Corners {
    pub fn new(tl: Radii, tr: Radii, bl: Radii, br: Radii) -> Self {
        Self {
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br
        }
    }
}