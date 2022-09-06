use layout::{flow::line_box::LineFragmentData, layout_box::LayoutBoxPtr};
use shared::{
    color::Color,
    primitive::{Corners, RRect, Rect, Size},
};
use style_types::{
    values::{color::Color as CSSColor, prelude::BorderStyle},
    Property, Value,
};

use crate::utils::{color_from_value, is_zero, to_radii};

pub struct RequestBuilder<'a> {
    boxes: Vec<PaintBox>,
    texts: Vec<PaintText>,
    root_element_use_body_background: bool,
    canvas_size: &'a Size,
}

pub struct PaintRequest {
    pub boxes: Vec<PaintBox>,
    pub texts: Vec<PaintText>,
}

pub struct PaintBox {
    pub rect: RectOrRRect,
    pub background_color: Color,
    pub borders: PaintBoxBorders,
    pub border_rect: Rect,
}

#[derive(Debug)]
pub struct PaintBoxBorders {
    pub top: Option<PaintBoxBorder>,
    pub right: Option<PaintBoxBorder>,
    pub bottom: Option<PaintBoxBorder>,
    pub left: Option<PaintBoxBorder>,
}

#[derive(Debug)]
pub struct PaintBoxBorder {
    pub style: BorderStyle,
    pub color: Color,
}

pub struct PaintText {
    pub content: String,
    pub font_size: f32,
    pub color: Color,
    pub rect: Rect,
}

#[derive(Debug)]
pub enum RectOrRRect {
    Rect(Rect),
    RRect(RRect),
}

impl<'a> RequestBuilder<'a> {
    pub fn new(canvas_size: &'a Size) -> Self {
        Self {
            boxes: Vec::new(),
            texts: Vec::new(),
            root_element_use_body_background: false,
            canvas_size,
        }
    }

    pub fn build(mut self, layout_box: &LayoutBoxPtr) -> PaintRequest {
        self.process(layout_box);
        PaintRequest {
            boxes: self.boxes,
            texts: self.texts,
        }
    }

    fn process(&mut self, layout_box: &LayoutBoxPtr) {
        if !layout_box.is_visible_in_scrolling_area() {
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

                        let box_is_visible = layout_box
                            .scrolling_containing_block()
                            .map(|block| text_rect.is_overlap_rect(&block.absolute_rect()))
                            .unwrap_or(true);

                        if !box_is_visible {
                            continue;
                        }

                        self.texts.push(PaintText {
                            content: content.to_string(),
                            color,
                            font_size,
                            rect: text_rect,
                        });
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

        let is_box_visible = layout_box
            .scrolling_containing_block()
            .map(|containing_block| rect.is_overlap_rect(&containing_block.absolute_rect()))
            .unwrap_or(true);

        if !is_box_visible {
            return;
        }

        let maybe_corners = self.compute_border_radius_corner(layout_box);

        let rect = if let Some(corners) = maybe_corners {
            RectOrRRect::RRect(RRect { rect, corners })
        } else {
            RectOrRRect::Rect(rect)
        };

        let borders = self.compute_borders(layout_box);
        let border_rect = layout_box.border_box_absolute();

        let paint_box = PaintBox {
            rect,
            background_color,
            borders,
            border_rect,
        };
        self.boxes.push(paint_box);
    }

    fn compute_borders(&self, layout_box: &LayoutBoxPtr) -> PaintBoxBorders {
        if layout_box.is_anonymous() {
            return PaintBoxBorders {
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
                    Value::BorderStyle(style) => Some(PaintBoxBorder {
                        color: color_from_value(&node.get_style(&Property::$color)),
                        style,
                    }),
                    _ => None,
                }
            };
        }

        PaintBoxBorders {
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

        fn get_rect_paint_box(rect: Rect, color: Color) -> PaintBox {
            let borders = PaintBoxBorders {
                top: None,
                right: None,
                bottom: None,
                left: None,
            };
            PaintBox {
                rect: RectOrRRect::Rect(rect.clone()),
                border_rect: rect,
                background_color: color,
                borders
            }
        }

        let gutter_paint_box = get_rect_paint_box(
            Rect::new(scroll_bar_x, scroll_bar_y, scroll_bar_width, scroll_bar_height),
            Color { r: 36, g: 36, b: 36, a: 255 }
        );

        let thumb_paint_box = get_rect_paint_box(
            Rect::new(scroll_bar_x, scroll_bar_thumb_y, scroll_bar_width, scroll_bar_thumb_height),
            Color { r: 173, g: 173, b: 173, a: 255 }
        );
        
        self.boxes.push(gutter_paint_box);
        self.boxes.push(thumb_paint_box);
    }
}
