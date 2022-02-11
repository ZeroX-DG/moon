use std::rc::Rc;

use shared::primitive::{Point, Size};
use style::property::Property;

use crate::{layout_box::LayoutBox, text::TextMeasure};

#[derive(Debug)]
pub struct LineFragment {
    pub data: LineFragmentData,
    pub offset: Point,
    pub size: Size,
}

#[derive(Debug)]
pub enum LineFragmentData {
    Box(Rc<LayoutBox>),
    Text(Rc<LayoutBox>, String),
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
        let box_model = child.box_model().borrow();
        let fragment = LineFragment::new_box(
            child.clone(),
            Point::new(self.size.width + box_model.margin.left, 0.),
            Size::new(fragment_width, fragment_height),
        );
        self.fragments.push(fragment);
        self.size.width += fragment_width + box_model.margin.right;
        self.size.height = f32::max(self.size.height, fragment_height);
    }

    pub fn add_text_fragment(
        &mut self,
        fragment_width: f32,
        fragment_height: f32,
        layout_box: Rc<LayoutBox>,
        text: String,
    ) {
        let fragment = LineFragment::new_text(
            layout_box,
            text,
            Point::new(self.size.width, 0.),
            Size::new(fragment_width, fragment_height),
        );
        self.fragments.push(fragment);
        self.size.width += fragment_width;
        self.size.height = f32::max(self.size.height, fragment_height);
    }

    pub fn dump(&self, level: usize) -> String {
        let mut result = String::new();

        let line_dimensions = format!("(w: {} | h: {})", self.size.width, self.size.height);

        result.push_str(&format!(
            "{}[LineBox]{}\n",
            "  ".repeat(level),
            line_dimensions
        ));

        for fragment in self.fragments.iter() {
            result.push_str(&fragment.dump(level + 1));
        }

        result
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

    pub fn new_text(layout_box: Rc<LayoutBox>, content: String, offset: Point, size: Size) -> Self {
        Self::new(LineFragmentData::Text(layout_box, content), offset, size)
    }

    pub fn dump(&self, level: usize) -> String {
        let fragment_type = match &self.data {
            LineFragmentData::Box(_) => "[Box Fragment]".to_string(),
            LineFragmentData::Text(_, content) => format!("[Text Fragment] {:?}", content),
        };

        let fragment_info = format!(
            "(x: {} | y: {} | w: {} | h: {})",
            self.offset.x, self.offset.y, self.size.width, self.size.height
        );

        let mut result = format!("{}{}{}\n", "  ".repeat(level), fragment_type, fragment_info);
        match &self.data {
            LineFragmentData::Box(node) => result.push_str(&node.dump(level + 1)),
            LineFragmentData::Text(_, _) => {}
        }
        result
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
        self.break_line_if_needed(layout_box.margin_box_width());

        self.current_line()
            .add_box_fragment(fragment_width, fragment_height, layout_box);
    }

    pub fn add_text_fragment(&mut self, layout_box: Rc<LayoutBox>, text: String) {
        let render_node = layout_box.render_node().unwrap();
        let font_size = render_node.get_style(&Property::FontSize).to_absolute_px();
        let mut text_measurer = TextMeasure::new();
        let text_size = text_measurer.measure(&text, font_size);
        let fragment_width = text_size.width;
        let fragment_height = text_size.height;
        self.break_line_if_needed(fragment_width);
        self.current_line()
            .add_text_fragment(fragment_width, fragment_height, layout_box, text);
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
