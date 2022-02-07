use std::rc::Rc;

use crate::command::{DisplayCommand, DrawCommand};
use crate::utils::color_from_value;
use layout::layout_box::LayoutBox;
use shared::primitive::*;
use style::property::Property;

pub fn paint_border(layout_node: Rc<LayoutBox>) -> Option<DisplayCommand> {
    if let Some(render_node) = &layout_node.render_node() {
        let border_top_color = render_node
            .get_style(&Property::BorderTopColor)
            .map(color_from_value);

        let border_bottom_color = render_node
            .get_style(&Property::BorderBottomColor)
            .map(color_from_value);

        let border_left_color = render_node
            .get_style(&Property::BorderLeftColor)
            .map(color_from_value);

        let border_right_color = render_node
            .get_style(&Property::BorderRightColor)
            .map(color_from_value);

        // TODO: support other border style other than solid
        let mut draw_commands = Vec::new();

        let box_model = layout_node.box_model().borrow();

        // TODO: Use trapezoid instead of rect
        if box_model.border.top > 0. {
            let rect = create_border_rect(layout_node.clone(), Edge::Top);
            draw_commands.push(DrawCommand::FillRect(rect, border_top_color));
        }

        if box_model.border.left > 0. {
            let rect = create_border_rect(layout_node.clone(), Edge::Left);
            draw_commands.push(DrawCommand::FillRect(rect, border_left_color));
        }

        if box_model.border.right > 0. {
            let rect = create_border_rect(layout_node.clone(), Edge::Right);
            draw_commands.push(DrawCommand::FillRect(rect, border_right_color));
        }

        if box_model.border.bottom > 0. {
            let rect = create_border_rect(layout_node.clone(), Edge::Bottom);
            draw_commands.push(DrawCommand::FillRect(rect, border_bottom_color));
        }

        return Some(DisplayCommand::GroupDraw(draw_commands));
    }
    None
}

fn create_border_rect(layout_node: Rc<LayoutBox>, edge: Edge) -> Rect {
    let border_box = layout_node.border_box_absolute();
    let box_model = layout_node.box_model().borrow();

    match edge {
        Edge::Top => Rect::new(
            border_box.x,
            border_box.y,
            border_box.width,
            box_model.border.top,
        ),
        Edge::Left => Rect::new(
            border_box.x,
            border_box.y,
            box_model.border.left,
            border_box.height,
        ),
        Edge::Right => Rect::new(
            border_box.x + border_box.width - box_model.border.right,
            border_box.y,
            box_model.border.right,
            border_box.height,
        ),
        Edge::Bottom => Rect::new(
            border_box.x,
            border_box.y + border_box.height - box_model.border.bottom,
            border_box.width,
            box_model.border.bottom,
        ),
    }
}
