use layout::layout_box::LayoutBox;
use super::Painter;
use super::rect::Rect;
use super::paint::{Paint, PaintStyle, PaintColor};
use style::value_processing::Property;
use style::values::color::Color;
use style::value_processing::Value;

#[derive(Debug)]
pub enum DisplayCommand {
    DrawRect(Rect, Paint)
}

pub type DisplayList = Vec<DisplayCommand>;

pub fn execute_display_command<P: Painter>(command: DisplayCommand, painter: &mut P) {
    match command {
        DisplayCommand::DrawRect(rect, paint) => painter.paint_rect(rect, paint)
    }
}

pub fn build_display_list(root: &LayoutBox) -> DisplayList {
    let mut display_list = Vec::new();

    render_layout_box(root, &mut display_list);

    display_list
}

fn render_layout_box(root: &LayoutBox, display_list: &mut DisplayList) {
    render_background(root, display_list);

    // TODO: render text
    for child in &root.children {
        render_layout_box(child, display_list);
    }
}

fn render_background(root: &LayoutBox, display_list: &mut DisplayList) {
    let render_node = root.render_node.borrow();
    let background = render_node.get_style(&Property::BackgroundColor);

    let paint = Paint {
        style: PaintStyle::Fill,
        color: style_color_to_paint_color(background.inner()).unwrap_or_default()
    };

    let rect_width = root.dimensions.content.width
        + root.dimensions.padding.left
        + root.dimensions.padding.right;

    let rect_height = root.dimensions.content.height
        + root.dimensions.padding.top
        + root.dimensions.padding.bottom;

    let rect = Rect {
        x: root.dimensions.content.x,
        y: root.dimensions.content.y,
        width: rect_width,
        height: rect_height
    };

    display_list.push(DisplayCommand::DrawRect(rect, paint));
}

fn style_color_to_paint_color(style_color: &Value) -> Option<PaintColor> {
    let color = match style_color {
        Value::Color(c) => c,
        _ => return None
    };

    match color {
        Color::Rgba(r, g, b, a) => {
            let alpha: u8 = a.as_u8();
            Some(PaintColor {
                r: r.as_u8(),
                g: g.as_u8(),
                b: b.as_u8(),
                a: alpha
            })
        }
        _ => None
    }
}
