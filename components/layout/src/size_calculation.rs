use super::layout_box::BoxType;
use super::layout_box::LayoutBox;
use super::utils::{is_replaced_element, Rect};
use style::value_processing::Property;
use style::value_processing::Value;
use style::values::display::Display;
use style::values::length::LengthUnit;

pub fn calculate_size(root: &mut LayoutBox, containing_block: &Rect) {
    match root.box_type {
        BoxType::Block => calculate_size_block_level(root, containing_block),
        _ => {}
    }
}

fn to_px(value: &Value, containing_block: &Rect) -> f32 {
    match value {
        Value::Length(l) => match l.unit {
            LengthUnit::Px => *l.value,
            _ => 0.0,
        },
        _ => 0.0,
    }
}

/// Calculate size for a box knowning the box is a block-level
fn calculate_size_block_level(layout_box: &mut LayoutBox, containing_block: &Rect) {
    let render_node = layout_box.render_node.borrow();
    let is_replaced_element = is_replaced_element(&render_node.node);
    let is_normal_flow = layout_box.parent_fmt_context.is_some();

    let width = render_node.get_style(&Property::Width);
    let margin_left = render_node.get_style(&Property::MarginLeft);
    let margin_right = render_node.get_style(&Property::MarginRight);
    let padding_left = render_node.get_style(&Property::PaddingLeft);
    let padding_right = render_node.get_style(&Property::PaddingRight);
    let border_left_width = render_node.get_style(&Property::BorderLeftWidth);
    let border_right_width = render_node.get_style(&Property::BorderRightWidth);

    if !is_replaced_element && is_normal_flow {
        let box_width = to_px(margin_left.inner(), containing_block)
            + to_px(border_left_width.inner(), containing_block)
            + to_px(padding_left.inner(), containing_block)
            + to_px(width.inner(), containing_block)
            + to_px(padding_right.inner(), containing_block)
            + to_px(border_right_width.inner(), containing_block)
            + to_px(margin_right.inner(), containing_block);
    }
}
