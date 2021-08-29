use crate::command::{DisplayCommand, DrawCommand};
use crate::primitive::{Corners, RRect, Radii, Rect};
use crate::{primitive::style_color_to_paint_color, utils::is_zero};
use layout::layout_box::LayoutNode;
use style::value_processing::{Property, Value};
use style::values::border_radius::BorderRadius;

pub fn paint_background(layout_node: &LayoutNode) -> Option<DisplayCommand> {
    if let Some(render_node) = &layout_node.render_node() {
        let render_node = render_node.borrow();
        let background = render_node.get_style(&Property::BackgroundColor);

        let border_top_left_radius = render_node.get_style(&Property::BorderTopLeftRadius);
        let border_bottom_left_radius = render_node.get_style(&Property::BorderBottomLeftRadius);
        let border_top_right_radius = render_node.get_style(&Property::BorderTopRightRadius);
        let border_bottom_right_radius = render_node.get_style(&Property::BorderBottomRightRadius);

        let color = style_color_to_paint_color(background.inner()).unwrap_or_default();

        let (x, y, width, height) = layout_node.dimensions().padding_box().into();

        let has_no_border_radius = is_zero(border_top_left_radius.inner())
            && is_zero(border_bottom_left_radius.inner())
            && is_zero(border_top_right_radius.inner())
            && is_zero(border_bottom_right_radius.inner());

        if has_no_border_radius {
            let rect = Rect {
                x,
                y,
                width,
                height,
            };

            return Some(DisplayCommand::Draw(DrawCommand::FillRect(rect, color)));
        } else {
            let border_box = layout_node.dimensions().border_box();

            let tl = to_radii(border_top_left_radius.inner(), border_box.width);
            let tr = to_radii(border_top_right_radius.inner(), border_box.width);
            let bl = to_radii(border_bottom_left_radius.inner(), border_box.width);
            let br = to_radii(border_bottom_right_radius.inner(), border_box.width);

            let rect = RRect {
                x,
                y,
                width,
                height,
                corners: Corners::new(tl, tr, bl, br),
            };

            return Some(DisplayCommand::Draw(DrawCommand::FillRRect(rect, color)));
        }
    }
    None
}

fn to_radii(value: &Value, width: f32) -> Radii {
    match value {
        Value::BorderRadius(BorderRadius(hr, vr)) => Radii::new(hr.to_px(width), vr.to_px(width)),
        _ => Radii::new(0.0, 0.0),
    }
}
