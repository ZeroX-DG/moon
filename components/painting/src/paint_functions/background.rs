use crate::color::style_color_to_paint_color;
use crate::command::DisplayCommand;
use crate::primitive::{Rect, RRect, Radius};
use crate::LayoutBox;
use style::value_processing::{Property, Value};

pub fn paint_background(layout_box: &LayoutBox) -> Option<DisplayCommand> {
    if let Some(render_node) = &layout_box.render_node {
        let render_node = render_node.borrow();
        let background = render_node.get_style(&Property::BackgroundColor);

        let border_top_left_radius = render_node.get_style(&Property::BorderTopLeftRadius);
        let border_bottom_left_radius = render_node.get_style(&Property::BorderBottomLeftRadius);
        let border_top_right_radius = render_node.get_style(&Property::BorderTopRightRadius);
        let border_bottom_right_radius = render_node.get_style(&Property::BorderBottomRightRadius);

        let color = style_color_to_paint_color(background.inner()).unwrap_or_default();

        let (x, y) = layout_box.dimensions.padding_box_position();
        let (width, height) = layout_box.dimensions.padding_box_size();

        let (tl, tr, bl, br) = match (
            border_top_left_radius.inner(),
            border_bottom_left_radius.inner(),
            border_top_right_radius.inner(),
            border_bottom_right_radius.inner(),
        ) {
            (
                Value::Length(tl),
                Value::Length(tr),
                Value::Length(bl),
                Value::Length(br),
            ) => {
                (*tl.value, *tr.value, *bl.value, *br.value)
            },
            _ => return None,
        };

        let has_no_border_radius = tl == 0. && tr == 0. && bl == 0. && br == 0.;

        if has_no_border_radius {
            let rect = Rect {
                x,
                y,
                width,
                height,
            };
    
            return Some(DisplayCommand::FillRect(rect, color));
        } else {
            let rect = RRect {
                x,
                y,
                width,
                height,
                radius: Radius::new(tl, tr, bl, br)
            };
    
            return Some(DisplayCommand::FillRRect(rect, color));
        }
    }
    None
}
