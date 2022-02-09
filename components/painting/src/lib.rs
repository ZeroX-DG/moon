mod gfx_painter;
mod utils;

use std::rc::Rc;

use layout::{layout_box::LayoutBox, flow::line_box::{LineBox, LineFragmentData}};

pub use gfx_painter::GfxPainter;
use shared::{primitive::{Size, Rect, Corners}, color::Color};
use style::property::Property;
use utils::{color_from_value, to_radii, is_zero};

pub struct Painter<G: GfxPainter> {
    gfx: G
}

impl<G: GfxPainter> Painter<G> {
    pub fn new(gfx: G) -> Self {
        Self {
            gfx
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.gfx.resize(size);
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
                    let mut background_rect = Rect::from((containing_block.absolute_location(), fragment.size.clone()));
                    background_rect.translate(fragment.offset.x, fragment.offset.y);
                    let background_color = color_from_value(&render_node.get_style(&Property::BackgroundColor));
                    let corners = self.compute_border_radius_corner(layout_box.clone());
                    self.paint_background(background_rect, background_color, corners);
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
        let background_rect = layout_box.padding_box_absolute();
        let background_color = color_from_value(&render_node.get_style(&Property::BackgroundColor));

        let corners = self.compute_border_radius_corner(layout_box);
        self.paint_background(background_rect, background_color, corners);
    }

    fn paint_background(&mut self, rect: Rect, color: Color, maybe_corners: Option<Corners>) {
        if let Some(corners) = maybe_corners {
            self.gfx.fill_rrect(shared::primitive::RRect { rect, corners }, color);
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

