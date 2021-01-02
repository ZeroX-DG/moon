use super::values::{Paint, PaintColor, PaintStyle};
use super::values::Rect;
use super::{DisplayCommand, DisplayList};
use layout::layout_box::LayoutBox;
use style::value_processing::Property;
use style::value_processing::Value;
use style::values::color::Color;

pub fn render_layout_box(root: &LayoutBox, display_list: &mut DisplayList) {
    render_background(root, display_list);

    // TODO: render text
    for child in &root.children {
        render_layout_box(child, display_list);
    }
}

fn render_background(root: &LayoutBox, display_list: &mut DisplayList) {
    if let Some(render_node) = &root.render_node {
        let render_node = render_node.borrow();
        let background = render_node.get_style(&Property::BackgroundColor);

        let paint = Paint {
            style: PaintStyle::Fill,
            color: style_color_to_paint_color(background.inner()).unwrap_or_default(),
        };

        let (x, y) = root.dimensions.padding_box_position();
        let (width, height) = root.dimensions.padding_box_size();

        let rect = Rect {
            x,
            y,
            width,
            height,
        };

        display_list.push(DisplayCommand::DrawRect(rect, paint));
    }
}

fn style_color_to_paint_color(style_color: &Value) -> Option<PaintColor> {
    let color = match style_color {
        Value::Color(c) => c,
        _ => return None,
    };

    match color {
        Color::Rgba(r, g, b, a) => {
            let alpha: u8 = a.as_u8();
            Some(PaintColor {
                r: r.as_u8(),
                g: g.as_u8(),
                b: b.as_u8(),
                a: alpha,
            })
        }
        _ => None,
    }
}
