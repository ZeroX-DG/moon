use crate::command::{DisplayCommand, DrawCommand};
use crate::utils::color_from_value;
use layout::layout_box::LayoutNode;
use style::property::Property;

pub fn paint_text(layout_node: &LayoutNode) -> Option<DisplayCommand> {
    if let Some(render_node) = &layout_node.render_node() {
        let render_node = render_node.borrow();

        if let Some(text) = render_node.node.as_text_opt() {
            let content = text.get_data();
            // TODO: support text bounds width & height
            let bounds = layout_node.dimensions().content_box();
            let color = render_node
                .get_style(&Property::Color)
                .map(color_from_value);
            let size = render_node.get_style(&Property::FontSize).to_absolute_px();
            return Some(DisplayCommand::Draw(DrawCommand::FillText(
                content, bounds, color, size,
            )));
        }
    }
    None
}
