use super::layout_box::LayoutBox;

#[derive(Debug, Clone)]
pub struct LineBox {
    fragments: Vec<*mut LayoutBox>,
    width: f32,
    height: f32,
}

impl LineBox {
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
            width: 0.,
            height: 0.,
        }
    }

    pub fn fragments(&self) -> Vec<&mut LayoutBox> {
        unsafe {
            self.fragments
                .iter()
                .map(|layout_box| layout_box.as_mut().unwrap())
                .collect()
        }
    }

    pub fn push(&mut self, layout_box: &mut LayoutBox) {
        let fragment_height = layout_box.dimensions.margin_box().height;
        let fragment_width = layout_box.dimensions.margin_box().width;

        if fragment_height > self.height {
            self.height = fragment_height;
        }

        self.width += fragment_width;

        self.fragments.push(layout_box);
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}
