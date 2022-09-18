use std::rc::Rc;

use shared::primitive::{Point, Size};
use style_types::{values::prelude::TextAlign, Property, Value};

use crate::{layout_box::LayoutBoxPtr, layout_context::LayoutContext};

#[derive(Debug)]
pub struct LineFragment {
    pub data: LineFragmentData,
    pub offset: Point,
    pub size: Size,
}

#[derive(Debug)]
pub enum LineFragmentData {
    Box(LayoutBoxPtr),
    Text(LayoutBoxPtr, String),
}

pub struct LineBoxBuilder {
    line_boxes: Vec<LineBox>,
    parent: LayoutBoxPtr,
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
        child: LayoutBoxPtr,
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
        layout_box: LayoutBoxPtr,
        text: String,
    ) {
        if !self.fragments.is_empty() {
            let last_fragment = self.fragments.last_mut().unwrap();

            if let LineFragmentData::Text(last_box, ref mut content) = &mut last_fragment.data {
                if Rc::ptr_eq(last_box, &layout_box) {
                    content.push_str(&text);
                    last_fragment.size.width += fragment_width;
                    self.size.width += fragment_width;
                    self.size.height = f32::max(self.size.height, fragment_height);
                    return;
                }
            }
        }
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

    pub fn new_box(layout_box: LayoutBoxPtr, offset: Point, size: Size) -> Self {
        Self::new(LineFragmentData::Box(layout_box), offset, size)
    }

    pub fn new_text(layout_box: LayoutBoxPtr, content: String, offset: Point, size: Size) -> Self {
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
    pub fn new(parent: LayoutBoxPtr) -> Self {
        Self {
            line_boxes: Vec::new(),
            parent,
            current_offset_y: 0.,
        }
    }

    pub fn finish(mut self, context: &mut LayoutContext) -> Vec<LineBox> {
        self.update_last_line(context);
        self.line_boxes
    }

    pub fn add_box_fragment(&mut self, context: &mut LayoutContext, layout_box: LayoutBoxPtr) {
        if let Some(node) = layout_box.node() {
            if let Some(element) = node.as_element_opt() {
                if element.tag_name() == "br" {
                    self.break_line(context);
                    self.update_last_line(context);
                    return;
                }
            }
        }

        let fragment_width = layout_box.content_size().width;
        let fragment_height = layout_box.content_size().height;
        self.break_line_if_needed(context, layout_box.margin_box_width());

        self.current_line()
            .add_box_fragment(fragment_width, fragment_height, layout_box);
    }

    pub fn add_text_fragment(
        &mut self,
        context: &mut LayoutContext,
        layout_box: LayoutBoxPtr,
        text: String,
    ) {
        let node = layout_box.node().unwrap();
        let font_size = node.get_style(&Property::FontSize).to_absolute_px();
        let text_size = context.measure_text(&text, font_size);
        let fragment_width = text_size.width;
        let fragment_height = text_size.height;
        self.break_line_if_needed(context, fragment_width);
        self.current_line()
            .add_text_fragment(fragment_width, fragment_height, layout_box, text);
    }

    fn break_line_if_needed(&mut self, context: &mut LayoutContext, next_fragment_width: f32) {
        if self.line_boxes.is_empty() {
            return;
        }
        let parent_width = self.parent.content_size().width;
        let new_line_box_width = self.current_line().size.width + next_fragment_width;

        let should_break = new_line_box_width > parent_width;

        if should_break {
            self.break_line(context);
        }
    }

    fn break_line(&mut self, context: &mut LayoutContext) {
        self.update_last_line(context);

        if let Some(last_line) = self.line_boxes.last() {
            self.current_offset_y += last_line.size.height;
        }

        self.line_boxes.push(LineBox::new());
    }

    fn update_last_line(&mut self, context: &mut LayoutContext) {
        if self.line_boxes.is_empty() {
            return;
        }

        let last_line = self.line_boxes.last_mut().unwrap();

        if last_line.fragments.is_empty() {
            let parent = self.parent.get_non_anonymous_parent();
            let font_size = parent
                .node()
                .unwrap()
                .get_style(&Property::FontSize)
                .to_absolute_px();
            let text_size = context.measure_text("H", font_size);

            last_line.size.height = text_size.height;
        }

        let mut x_offset = last_line
            .fragments
            .iter()
            .map(|fragment| fragment.offset.x)
            .fold(f32::INFINITY, |a, b| a.min(b));

        let remaining_space = self.parent.content_size().width - last_line.size.width;

        if let Some(node) = self.parent.node() {
            match node.get_style(&Property::TextAlign) {
                Value::TextAlign(TextAlign::Center) => {
                    x_offset += remaining_space / 2.;
                }
                _ => {}
            }
        }

        for fragment in &mut last_line.fragments {
            let mut used_offset = Point::new(fragment.offset.x, self.current_offset_y);
            used_offset.translate(x_offset, 0.);
            fragment.set_offset(used_offset);
        }
    }

    fn current_line(&mut self) -> &mut LineBox {
        if self.line_boxes.is_empty() {
            self.line_boxes.push(LineBox::new());
        }
        self.line_boxes.last_mut().unwrap()
    }
}
