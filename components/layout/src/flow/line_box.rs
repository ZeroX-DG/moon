use std::rc::Rc;

use crate::layout_box::LayoutBox;

#[derive(Debug)]
pub enum LineFragment {
    Box(Rc<LayoutBox>)
}

pub struct LineBoxBuilder {
    line_boxes: Vec<LineBox>,
    parent: Rc<LayoutBox>
}

#[derive(Debug)]
pub struct LineBox {
    fragments: Vec<LineFragment>,
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

    pub fn add_box_fragment(&mut self, child: Rc<LayoutBox>) {
        let child_size = child.dimensions().margin_box();
        self.fragments.push(LineFragment::Box(child));
        self.width += child_size.width;
        self.height = if self.height > child_size.height {
            self.height
        } else {
            child_size.height
        };
    }

    pub fn fragments(&self) -> &[LineFragment] {
        &self.fragments
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

impl LineBoxBuilder {
    pub fn new(parent: Rc<LayoutBox>) -> Self {
        Self {
            line_boxes: Vec::new(),
            parent
        }
    }

    pub fn add_box_fragment(&mut self, layout_box: Rc<LayoutBox>) {
        let fragment_width = layout_box.dimensions().margin_box().width;
        self.add_fragment(fragment_width, LineFragment::Box(layout_box));
    }

    fn add_fragment(&mut self, fragment_width: f32, fragment: LineFragment) {
        let parent_width = self.parent.dimensions().content_box().width;
        let new_line_box_width = self.current_line().width() + fragment_width;
        if new_line_box_width > parent_width {
            self.line_boxes.push(LineBox::new());
        }
        match fragment {
            LineFragment::Box(layout_box) => self.current_line().add_box_fragment(layout_box),
        }
    }

    pub fn finish(self) -> Vec<LineBox> {
        self.line_boxes
    }

    fn current_line(&mut self) -> &mut LineBox {
        if self.line_boxes.is_empty() {
            self.line_boxes.push(LineBox::new());
        }
        self.line_boxes.last_mut().unwrap()
    }
}