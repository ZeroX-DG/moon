mod utils;

use std::rc::Rc;

use layout::{
    flow::line_box::{LineBox, LineFragmentData},
    layout_box::LayoutBox,
};

use gfx::Graphics;
use shared::{
    color::Color,
    primitive::{Corners, Rect, Size},
};
use style::{property::Property, value::Value, values::color::Color as CSSColor};
use utils::{color_from_value, is_zero, to_radii};

pub struct Painter<G: Graphics> {
    gfx: G,
    root_element_use_body_background: bool,
    canvas_size: Size,
}

impl<G: Graphics> Painter<G> {
    pub fn new(gfx: G) -> Self {
        Self {
            gfx,
            root_element_use_body_background: false,
            canvas_size: Size::default(),
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.gfx.resize(size.clone());
        self.canvas_size = size;
    }

    pub async fn output(&mut self) -> Vec<u8> {
        let result = self.gfx.output().await;
        result
    }

    pub fn paint(&mut self, layout_box: Rc<LayoutBox>) {
        self.paint_box_background(layout_box.clone());

        if layout_box.children_are_inline() {
            for line in layout_box.lines().borrow().iter() {
                self.paint_line(layout_box.clone(), line);
            }
            return;
        }

        for child in layout_box.children().iter() {
            self.paint(child.clone());
        }
    }

    fn paint_line(&mut self, containing_block: Rc<LayoutBox>, line: &LineBox) {
        for fragment in &line.fragments {
            match &fragment.data {
                LineFragmentData::Box(layout_box) if !layout_box.is_anonymous() => {
                    let render_node = layout_box.render_node().unwrap();
                    let mut background_rect =
                        Rect::from((containing_block.absolute_location(), fragment.size.clone()));
                    background_rect.translate(fragment.offset.x, fragment.offset.y);
                    let background_color =
                        color_from_value(&render_node.get_style(&Property::BackgroundColor));
                    let corners = self.compute_border_radius_corner(layout_box.clone());
                    self.paint_background(background_rect, background_color, corners);
                }
                LineFragmentData::Text(layout_box, content) => {
                    let render_node = layout_box.render_node().unwrap();
                    let mut text_rect =
                        Rect::from((containing_block.absolute_location(), fragment.size.clone()));
                    text_rect.translate(fragment.offset.x, fragment.offset.y);
                    let text_color = color_from_value(&render_node.get_style(&Property::Color));
                    let font_size = render_node.get_style(&Property::FontSize).to_absolute_px();
                    self.gfx
                        .fill_text(content.clone(), text_rect, text_color, font_size);
                }
                _ => {}
            }
        }
    }

    fn paint_box_background(&mut self, layout_box: Rc<LayoutBox>) {
        if layout_box.is_anonymous() {
            return;
        }

        let render_node = layout_box.render_node().unwrap();
        let mut background_rect = layout_box.padding_box_absolute();
        let background_color = color_from_value(&render_node.get_style(&Property::BackgroundColor));

        if layout_box.is_root_element() {
            self.root_element_use_body_background = {
                match render_node.get_style(&Property::BackgroundColor).inner() {
                    Value::Color(CSSColor::Transparent) => true,
                    _ => false,
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
                background_rect =
                    Rect::new(0., 0., self.canvas_size.width, self.canvas_size.height);
            }
        }

        let corners = self.compute_border_radius_corner(layout_box);
        self.paint_background(background_rect, background_color, corners);
    }

    fn paint_background(&mut self, rect: Rect, color: Color, maybe_corners: Option<Corners>) {
        if let Some(corners) = maybe_corners {
            self.gfx
                .fill_rrect(shared::primitive::RRect { rect, corners }, color);
        } else {
            self.gfx.fill_rect(rect, color);
        }
    }

    fn compute_border_radius_corner(&self, layout_box: Rc<LayoutBox>) -> Option<Corners> {
        if layout_box.is_anonymous() {
            return None;
        }
        let render_node = layout_box.render_node().unwrap();
        let border_top_left_radius = render_node.get_style(&Property::BorderTopLeftRadius);
        let border_bottom_left_radius = render_node.get_style(&Property::BorderBottomLeftRadius);
        let border_top_right_radius = render_node.get_style(&Property::BorderTopRightRadius);
        let border_bottom_right_radius = render_node.get_style(&Property::BorderBottomRightRadius);

        let has_no_border_radius = is_zero(border_top_left_radius.inner())
            && is_zero(border_bottom_left_radius.inner())
            && is_zero(border_top_right_radius.inner())
            && is_zero(border_bottom_right_radius.inner());

        if has_no_border_radius {
            return None;
        }

        let border_box = layout_box.border_box_absolute();

        let tl = to_radii(border_top_left_radius.inner(), border_box.width);
        let tr = to_radii(border_top_right_radius.inner(), border_box.width);
        let bl = to_radii(border_bottom_left_radius.inner(), border_box.width);
        let br = to_radii(border_bottom_right_radius.inner(), border_box.width);

        Some(Corners::new(tl, tr, bl, br))
    }
}
