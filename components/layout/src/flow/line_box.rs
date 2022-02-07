use std::rc::Rc;

use shared::primitive::{Point, Size};

use crate::layout_box::LayoutBox;

#[derive(Debug)]
pub struct LineFragment {
    pub data: LineFragmentData,
    pub offset: Point,
    pub size: Size
}

#[derive(Debug)]
pub enum LineFragmentData {
    Box(Rc<LayoutBox>)
}

pub struct LineBoxBuilder {
    line_boxes: Vec<LineBox>,
    parent: Rc<LayoutBox>
}

#[derive(Debug)]
pub struct LineBox {
    fragments: Vec<LineFragment>,
    pub size: Size
}

impl LineBox {
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
            size: Size::new(0., 0.)
        }
    }

    pub fn add_box_fragment(&mut self, child: Rc<LayoutBox>) {
        let child_size = child.content_size();
        self.fragments.push(LineFragment::new_box(child));
        self.size.width += child_size.width;
        self.size.height = if self.size.height > child_size.height {
            self.size.height
        } else {
            child_size.height
        };
    }

    pub fn fragments(&self) -> &[LineFragment] {
        &self.fragments
    }
}

impl LineFragment {
    pub fn new(data: LineFragmentData) -> Self {
        Self {
            data,
            offset: Point::new(0., 0.),
            size: Size::new(0., 0.)
        }
    }

    pub fn new_box(layout_box: Rc<LayoutBox>) -> Self {
        Self::new(LineFragmentData::Box(layout_box))
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
        let fragment_width = layout_box.content_size().width;
        self.add_fragment(fragment_width, LineFragment::new_box(layout_box));
    }

    fn add_fragment(&mut self, fragment_width: f32, fragment: LineFragment) {
        let parent_width = self.parent.content_size().width;
        let new_line_box_width = self.current_line().size.width + fragment_width;
        if new_line_box_width > parent_width {
            self.line_boxes.push(LineBox::new());
        }
        match fragment.data {
            LineFragmentData::Box(layout_box) => self.current_line().add_box_fragment(layout_box),
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
