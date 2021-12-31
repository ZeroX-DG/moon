use crate::command::{DisplayCommand, DrawCommand};
use crate::utils::color_from_value;
use layout::layout_box::LayoutNode;
use shared::primitive::*;
use style::property::Property;

pub fn paint_border(layout_node: &LayoutNode) -> Option<DisplayCommand> {
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

        // TODO: Use trapezoid instead of rect
        if layout_node.dimensions().border.top > 0. {
            let rect = create_border_rect(layout_node, Edge::Top);
            draw_commands.push(DrawCommand::FillRect(rect, border_top_color));
        }

        if layout_node.dimensions().border.left > 0. {
            let rect = create_border_rect(layout_node, Edge::Left);
            draw_commands.push(DrawCommand::FillRect(rect, border_left_color));
        }

        if layout_node.dimensions().border.right > 0. {
            let rect = create_border_rect(layout_node, Edge::Right);
            draw_commands.push(DrawCommand::FillRect(rect, border_right_color));
        }

        if layout_node.dimensions().border.bottom > 0. {
            let rect = create_border_rect(layout_node, Edge::Bottom);
            draw_commands.push(DrawCommand::FillRect(rect, border_bottom_color));
        }

        return Some(DisplayCommand::GroupDraw(draw_commands));
    }
    None
}

fn create_border_rect(layout_node: &LayoutNode, edge: Edge) -> Rect {
    let border_box = layout_node.dimensions().border_box();

    match edge {
        Edge::Top => Rect::new(
            border_box.x,
            border_box.y,
            border_box.width,
            layout_node.dimensions().border.top,
        ),
        Edge::Left => Rect::new(
            border_box.x,
            border_box.y,
            layout_node.dimensions().border.left,
            border_box.height,
        ),
        Edge::Right => Rect::new(
            border_box.x + border_box.width - layout_node.dimensions().border.right,
            border_box.y,
            layout_node.dimensions().border.right,
            border_box.height,
        ),
        Edge::Bottom => Rect::new(
            border_box.x,
            border_box.y + border_box.height - layout_node.dimensions().border.bottom,
            border_box.width,
            layout_node.dimensions().border.bottom,
        ),
    }
}
