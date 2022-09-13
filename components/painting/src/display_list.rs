use layout::{layout_box::LayoutBoxPtr, flow::line_box::LineFragmentData};
use shared::{color::Color, primitive::{Rect, RRect, Size, Corners}};
use style_types::{
    values::{color::Color as CSSColor, prelude::BorderStyle},
    Property, Value,
};

use crate::utils::{color_from_value, is_zero, to_radii};

pub struct DisplayList(Vec<Command>);

pub enum Command {
    FillRect(Rect, Color),
    FillRRect(RRect, Color),
    FillBorder(Rect, Rect, Borders),
    FillText(String, Rect, Color, f32),
}

pub struct Borders {
    pub top: Option<Border>,
    pub right: Option<Border>,
    pub bottom: Option<Border>,
    pub left: Option<Border>,
}

pub struct Border {
    pub style: BorderStyle,
    pub color: Color,
}

pub struct DisplayListBuilder<'a> {
    canvas_size: &'a Size,
    display_list: DisplayList,
    root_element_use_body_background: bool,
}

impl<'a> DisplayListBuilder<'a> {
    pub fn new(canvas_size: &'a Size) -> Self {
        Self {
            canvas_size,
            display_list: DisplayList::new(),
            root_element_use_body_background: false,
        }
    }

    pub fn build(mut self, layout_box: &LayoutBoxPtr) -> DisplayList {
        self.process(layout_box);
        self.display_list
    }

    fn process(&mut self, layout_box: &LayoutBoxPtr) {
        if !layout_box.is_visible_for_painting(None) {
            return;
        }

        self.build_paint_boxes(layout_box, None);

        if layout_box.is_block() && layout_box.children_are_inline() {
            self.process_lines(layout_box);
        }

        layout_box.for_each_child(|child| self.process(&LayoutBoxPtr(child)));
        self.build_paint_box_for_vertical_scroll_bar(layout_box);
    }

    fn process_lines(&mut self, containing_block: &LayoutBoxPtr) {
        assert!(containing_block.is_block() && containing_block.children_are_inline());

        for line in containing_block.lines().borrow().iter() {
            for fragment in &line.fragments {
                match &fragment.data {
                    LineFragmentData::Box(layout_box) if !layout_box.is_anonymous() => {
                        let mut rect = Rect::from((
                            containing_block.absolute_location(),
                            fragment.size.clone(),
                        ));
                        rect.translate(fragment.offset.x, fragment.offset.y);
                        self.build_paint_boxes(layout_box, Some(rect));
                    }
                    LineFragmentData::Text(layout_box, content) => {
                        let node = layout_box.node().unwrap();
                        let mut text_rect = Rect::from((
                            containing_block.absolute_location(),
                            fragment.size.clone(),
                        ));
                        text_rect.translate(fragment.offset.x, fragment.offset.y);
                        let color = color_from_value(&node.get_style(&Property::Color));
                        let font_size = node.get_style(&Property::FontSize).to_absolute_px();

                        let box_is_visible = layout_box.is_visible_for_painting(Some(&text_rect));

                        if !box_is_visible {
                            continue;
                        }

                        self.display_list.fill_text(content.to_string(), text_rect, color, font_size);
                    }
                    _ => {}
                }
            }
        }
    }

    fn build_paint_boxes(
        &mut self,
        layout_box: &LayoutBoxPtr,
        override_rect: Option<Rect>,
    ) {
        if layout_box.is_anonymous() {
            return;
        }

        let node = layout_box.node().unwrap();
        let mut rect = override_rect.unwrap_or(layout_box.padding_box_absolute());
        let background_color = color_from_value(&node.get_style(&Property::BackgroundColor));

        if layout_box.is_root_element() {
            self.root_element_use_body_background = {
                if let Value::Color(CSSColor::Transparent) =
                    node.get_style(&Property::BackgroundColor)
                {
                    true
                } else {
                    false
                }
            };

            if self.root_element_use_body_background {
                // Delegate the rendering to the body element
                return;
            }
        }

        if layout_box.is_body_element() && self.root_element_use_body_background {
            // Render the canvas for the root element if has been delegated.
            if self.root_element_use_body_background {
                rect = Rect::new(0., 0., self.canvas_size.width, self.canvas_size.height);
            }
        }

        let is_box_visible = layout_box.is_visible_for_painting(Some(&rect));

        if !is_box_visible {
            return;
        }

        let maybe_corners = self.compute_border_radius_corner(layout_box);
        let borders = self.compute_borders(layout_box);
        let border_rect = layout_box.border_box_absolute();

        match maybe_corners {
            Some(corners) => {
                self.display_list.fill_rrect(RRect::from((rect, corners)), background_color);
            }
            _ => {
                self.display_list.fill_rect(rect.clone(), background_color);
                self.display_list.fill_borders(rect, border_rect, borders);
            }
        }

    }

    fn compute_borders(&self, layout_box: &LayoutBoxPtr) -> Borders { 
        if layout_box.is_anonymous() {
            return Borders {
                top: None,
                right: None,
                bottom: None,
                left: None,
            };
        }
        let node = layout_box.node().unwrap();
        macro_rules! compute_border {
            ($style:ident, $color:ident) => {
                match node.get_style(&Property::$style) {
                    Value::BorderStyle(BorderStyle::None) => None,
                    Value::BorderStyle(style) => Some(Border {
                        color: color_from_value(&node.get_style(&Property::$color)),
                        style,
                    }),
                    _ => None,
                }
            };
        }

        Borders {
            top: compute_border!(BorderTopStyle, BorderTopColor),
            right: compute_border!(BorderRightStyle, BorderRightColor),
            bottom: compute_border!(BorderBottomStyle, BorderBottomColor),
            left: compute_border!(BorderLeftStyle, BorderLeftColor),
        }
    }

    fn compute_border_radius_corner(&self, layout_box: &LayoutBoxPtr) -> Option<Corners> {
        if layout_box.is_anonymous() {
            return None;
        }
        let node = layout_box.node().unwrap();
        let border_top_left_radius = node.get_style(&Property::BorderTopLeftRadius);
        let border_bottom_left_radius = node.get_style(&Property::BorderBottomLeftRadius);
        let border_top_right_radius = node.get_style(&Property::BorderTopRightRadius);
        let border_bottom_right_radius = node.get_style(&Property::BorderBottomRightRadius);

        let has_no_border_radius = is_zero(&border_top_left_radius)
            && is_zero(&border_bottom_left_radius)
            && is_zero(&border_top_right_radius)
            && is_zero(&border_bottom_right_radius);

        if has_no_border_radius {
            return None;
        }

        let border_box = layout_box.border_box_absolute();

        let font_size = node.get_style(&Property::FontSize).to_absolute_px();

        let tl = to_radii(&border_top_left_radius, border_box.width, font_size);
        let tr = to_radii(&border_top_right_radius, border_box.width, font_size);
        let bl = to_radii(&border_bottom_left_radius, border_box.width, font_size);
        let br = to_radii(&border_bottom_right_radius, border_box.width, font_size);

        Some(Corners::new(tl, tr, bl, br))
    }

    fn build_paint_box_for_vertical_scroll_bar(&mut self, layout_box: &LayoutBoxPtr) {
        if !layout_box.scrollable() {
            return;
        }

        let container_rect = layout_box.padding_box_absolute();
        let container_scroll_height = layout_box.scroll_height();
        let scroll_bar_width = layout_box.scrollbar_width();
        let scroll_bar_height = container_rect.height;
        let scroll_bar_x = container_rect.x + container_rect.width;
        let scroll_bar_y = container_rect.y;

        // Thanks to Huy Nguyen
        let scroll_bar_thumb_height = container_rect.height * (container_rect.height / container_scroll_height);
        let scroll_bar_thumb_y = container_rect.y + layout_box.scroll_top() * (container_rect.height / container_scroll_height);

        // Gutter
        self.display_list.fill_rect(
            Rect::new(scroll_bar_x, scroll_bar_y, scroll_bar_width, scroll_bar_height),
            Color { r: 36, g: 36, b: 36, a: 255 }
        );

        // Thumb
        self.display_list.fill_rect(
            Rect::new(scroll_bar_x, scroll_bar_thumb_y, scroll_bar_width, scroll_bar_thumb_height),
            Color { r: 173, g: 173, b: 173, a: 255 }
        );
        
    }
}

impl DisplayList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let command = Command::FillRect(rect, color);
        self.0.push(command);
    }

    pub fn fill_rrect(&mut self, rect: RRect, color: Color) {
        let command = Command::FillRRect(rect, color);
        self.0.push(command);
    }

    pub fn fill_borders(&mut self, rect: Rect, border_rect: Rect, borders: Borders) {
        let command = Command::FillBorder(rect, border_rect, borders);
        self.0.push(command);
    }

    pub fn fill_text(&mut self, content: String, rect: Rect, color: Color, font_size: f32) {
        let command = Command::FillText(content, rect, color, font_size);
        self.0.push(command);
    }
    pub fn commands(self) -> Vec<Command> {
        self.0
    }
}

