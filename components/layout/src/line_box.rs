use super::layout_box::LayoutBox;
use super::box_model::Rect;

#[derive(Debug, Clone)]
pub struct LineBox {
    children: Vec<*mut LayoutBox>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    offset_x: f32,
}

impl LineBox {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            children: Vec::new(),
            x,
            y,
            width: 0.,
            height: 0.,
            offset_x: 0.,
        }
    }

    pub fn push(&mut self, layout_box: &mut LayoutBox) {
        self.children.push(layout_box);
    }

    pub fn get_rect(&self) -> Rect {
        return Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height
        }
    }
}

