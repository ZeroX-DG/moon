use super::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RRect {
    pub rect: Rect,
    pub corners: Corners,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Corners {
    pub top_left: Radii,
    pub top_right: Radii,
    pub bottom_left: Radii,
    pub bottom_right: Radii,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Radii(f32, f32);

impl RRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32, corners: Corners) -> Self {
        Self {
            rect: Rect {
                x,
                y,
                width: w,
                height: h,
            },
            corners,
        }
    }
}

impl From<(Rect, Corners)> for RRect {
    fn from((rect, corners): (Rect, Corners)) -> Self {
        Self {
            rect,
            corners
        }
    }
}

impl std::ops::Deref for RRect {
    type Target = Rect;
    fn deref(&self) -> &Self::Target {
        &self.rect
    }
}

impl Corners {
    pub fn new(tl: Radii, tr: Radii, bl: Radii, br: Radii) -> Self {
        Self {
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br,
        }
    }
}

impl Radii {
    pub fn new(h: f32, v: f32) -> Self {
        Self(h, v)
    }

    pub fn vertical_r(&self) -> f32 {
        self.1
    }

    pub fn horizontal_r(&self) -> f32 {
        self.0
    }
}
