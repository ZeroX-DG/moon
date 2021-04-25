use super::layout_box::LayoutBox;

#[derive(Debug, Clone)]
pub struct LineBox {
    children: Vec<*const LayoutBox>
}

impl LineBox {
    pub fn new() -> Self {
        Self {
            children: Vec::new()
        }
    }

    pub fn push(&mut self, layout_box: &LayoutBox) {
        self.children.push(layout_box);
    }
}

