use crate::command::{DisplayCommand, DrawCommand};
use layout::layout_box::LayoutNode;
use style::property::Property;
use crate::utils::color_from_value;

pub fn paint_text(layout_node: &LayoutNode) -> Option<DisplayCommand> {
    if let Some(render_node) = &layout_node.render_node() {
        let render_node = render_node.borrow();
        let node = render_node.node.borrow();

        if let Some(text) = node.as_text_opt() {
            let content = text.get_data();
            // TODO: support text bounds width & height
            let mut bounds = layout_node.dimensions().content_box();
            bounds.width = 100.;
            bounds.height = 100.;
            let color = render_node.get_style(&Property::Color).map(color_from_value);
            // TODO: support font size
            let size = 20.0;
            return Some(DisplayCommand::Draw(DrawCommand::FillText(content, bounds, color, size)));
        }
    }
    None
}
