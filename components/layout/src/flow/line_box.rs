use std::rc::Rc;

use shared::primitive::{Point, Size};

use crate::layout_box::LayoutBox;

#[derive(Debug)]
pub struct LineFragment {
    pub data: LineFragmentData,
    pub offset: Point,
    pub size: Size,
}

#[derive(Debug)]
pub enum LineFragmentData {
    Box(Rc<LayoutBox>),
}

pub struct LineBoxBuilder {
    line_boxes: Vec<LineBox>,
    parent: Rc<LayoutBox>,
    current_offset_y: f32,
}

#[derive(Debug)]
pub struct LineBox {
    pub fragments: Vec<LineFragment>,
    pub size: Size,
}

impl LineBox {
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
            size: Size::new(0., 0.),
        }
    }

    pub fn add_box_fragment(
        &mut self,
        fragment_width: f32,
        fragment_height: f32,
        child: Rc<LayoutBox>,
    ) {
        let mut fragment = LineFragment::new_box(
            child.clone(),
            Point::new(self.size.width, 0.),
            Size::new(fragment_width, fragment_height),
        );
        fragment.offset.x = self.size.width;
        self.fragments.push(fragment);
        self.size.width += fragment_width;
        self.size.height = f32::max(self.size.height, fragment_height);
    }
}

impl LineFragment {
    pub fn new(data: LineFragmentData, offset: Point, size: Size) -> Self {
        Self { data, offset, size }
    }

    pub fn set_offset(&mut self, offset: Point) {
        self.offset = offset;
    }

    pub fn new_box(layout_box: Rc<LayoutBox>, offset: Point, size: Size) -> Self {
        Self::new(LineFragmentData::Box(layout_box), offset, size)
    }
}

impl LineBoxBuilder {
    pub fn new(parent: Rc<LayoutBox>) -> Self {
        Self {
            line_boxes: Vec::new(),
            parent,
            current_offset_y: 0.,
        }
    }

    pub fn finish(mut self) -> Vec<LineBox> {
        self.update_last_line();
        self.line_boxes
    }

    pub fn add_box_fragment(&mut self, layout_box: Rc<LayoutBox>) {
        let fragment_width = layout_box.content_size().width;
        let fragment_height = layout_box.content_size().height;
        self.break_line_if_needed(fragment_width);

        self.current_line()
            .add_box_fragment(fragment_width, fragment_height, layout_box);
    }

    fn break_line_if_needed(&mut self, next_fragment_width: f32) {
        if self.line_boxes.is_empty() {
            return;
        }
        let parent_width = self.parent.content_size().width;
        let new_line_box_width = self.current_line().size.width + next_fragment_width;

        let should_break = new_line_box_width > parent_width;

        if should_break {
            self.break_line();
        }
    }

    fn break_line(&mut self) {
        self.update_last_line();

        let last_line_height = self.line_boxes.last().unwrap().size.height;
        self.current_offset_y += last_line_height;

        self.line_boxes.push(LineBox::new());
    }

    fn update_last_line(&mut self) {
        if self.line_boxes.is_empty() {
            return;
        }

        let last_line = self.line_boxes.last_mut().unwrap();

        for fragment in &mut last_line.fragments {
            fragment.set_offset(Point::new(fragment.offset.x, self.current_offset_y));
        }
    }

    fn current_line(&mut self) -> &mut LineBox {
        if self.line_boxes.is_empty() {
            self.line_boxes.push(LineBox::new());
        }
        self.line_boxes.last_mut().unwrap()
    }
}
