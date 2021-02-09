use crate::color::style_color_to_paint_color;
use crate::command::DisplayCommand;
use crate::rect::Rect;
use crate::LayoutBox;
use style::value_processing::Property;

pub fn paint_background(layout_box: &LayoutBox) -> Option<DisplayCommand> {
    if let Some(render_node) = &layout_box.render_node {
        let render_node = render_node.borrow();
        let background = render_node.get_style(&Property::BackgroundColor);

        let color = style_color_to_paint_color(background.inner()).unwrap_or_default();

        let (x, y) = layout_box.dimensions.padding_box_position();
        let (width, height) = layout_box.dimensions.padding_box_size();

        let rect = Rect {
            x,
            y,
            width,
            height,
        };

        return Some(DisplayCommand::FillRect(rect, color));
    }
    None
}
