use super::layout_box::BoxType;
use super::layout_box::LayoutBox;
use super::box_model::{BoxComponent, Edge};
use super::utils::{is_replaced_element, Rect};
use style::value_processing::Property;
use style::value_processing::Value;
use style::value_processing::ValueRef;
use style::values::length::{Length, LengthUnit};

pub fn compute_size(root: &mut LayoutBox, containing_block: &Rect) {
    match root.box_type {
        BoxType::Block => compute_width_block_level(root, containing_block),
        _ => {}
    }
}

fn to_px(value: &Value, containing: f32) -> f32 {
    match value {
        Value::Length(l) => match l.unit {
            LengthUnit::Px => *l.value,
            _ => 0.0,
        },
        Value::Percentage(p) => p.to_px(containing),
        _ => 0.0,
    }
}

/// Compute width for a box knowning the box is a block-level
fn compute_width_block_level(layout_box: &mut LayoutBox, containing_block: &Rect) {
    let render_node = layout_box.render_node.borrow();
    let is_replaced_element = is_replaced_element(&render_node.node);
    let is_normal_flow = layout_box.parent_fmt_context.is_some();
    let zero_length = ValueRef::new(Value::Length(Length::zero()));

    let mut width = render_node.get_style(&Property::Width);
    let mut margin_left = render_node.get_style(&Property::MarginLeft);
    let mut margin_right = render_node.get_style(&Property::MarginRight);
    let padding_left = render_node.get_style(&Property::PaddingLeft);
    let padding_right = render_node.get_style(&Property::PaddingRight);
    let border_left_width = render_node.get_style(&Property::BorderLeftWidth);
    let border_right_width = render_node.get_style(&Property::BorderRightWidth);

    // 10.3.3 Block-level, non-replaced elements in normal flow
    if !is_replaced_element && is_normal_flow {
        let box_width = to_px(margin_left.inner(), containing_block.width)
            + to_px(border_left_width.inner(), containing_block.width)
            + to_px(padding_left.inner(), containing_block.width)
            + to_px(width.inner(), containing_block.width)
            + to_px(padding_right.inner(), containing_block.width)
            + to_px(border_right_width.inner(), containing_block.width)
            + to_px(margin_right.inner(), containing_block.width);

        if *width.inner() != Value::Auto && box_width > containing_block.width {
            // If 'width' is not 'auto' and 'border-left-width' + 'padding-left'
            // + 'width' + 'padding-right' + 'border-right-width' (plus any of
            // 'margin-left' or 'margin-right' that are not 'auto') is larger
            // than the width of the containing block, then any 'auto' values
            // for 'margin-left' or 'margin-right' are, for the following rules,
            // treated as zero.
            if *margin_left.inner() == Value::Auto {
                margin_left = zero_length.clone();
            }

            if *margin_right.inner() == Value::Auto {
                margin_right = zero_length.clone();
            }
        }

        // the difference between the containing block width
        // and the block width. This could result in negative
        // if the block width is greater than the containing block
        let difference = containing_block.width - box_width;
        match (
            *width.inner() == Value::Auto,
            *margin_left.inner() == Value::Auto,
            *margin_right.inner() == Value::Auto,
        ) {
            // If all of the above have a computed value other than 'auto',
            // the values are said to be "over-constrained" and one of the
            // used values will have to be different from its computed value.
            (false, false, false) => {
                // TODO: support for rtl direction of the containing block
                // We must somehow structure the containing block rect to
                // hold the direction too
                let abs_margin_right = to_px(margin_right.inner(), containing_block.width);
                margin_right =
                    ValueRef::new(Value::Length(Length::new_px(abs_margin_right + difference)));
            }
            // If there is exactly one value specified as 'auto', its
            // used value follows from the equality.
            (false, true, false) => {
                margin_left = ValueRef::new(Value::Length(Length::new_px(difference)))
            }
            (false, false, true) => {
                margin_right = ValueRef::new(Value::Length(Length::new_px(difference)))
            }
            // If 'width' is set to 'auto', any other 'auto' values become '0'
            // and 'width' follows from the resulting equality.
            (true, _, _) => {
                if *margin_left.inner() == Value::Auto {
                    margin_left = zero_length.clone();
                }

                if *margin_right.inner() == Value::Auto {
                    margin_right = zero_length.clone();
                }

                if difference > 0.0 {
                    width = ValueRef::new(Value::Length(Length::new_px(difference)));
                } else {
                    // the block width is greater than the containing block
                    // adjust margin right for ltr direction
                    let abs_margin_right = to_px(margin_right.inner(), containing_block.width);
                    width = zero_length.clone();
                    margin_right =
                        ValueRef::new(Value::Length(Length::new_px(abs_margin_right + difference)));
                }
            }
            // If both 'margin-left' and 'margin-right' are 'auto', their
            // used values are equal. This horizontally centers the element 
            // with respect to the edges of the containing block. 
            (false, true, true) => {
                let half_difference = ValueRef::new(Value::Length(Length::new_px(difference / 2.0)));
                margin_left = half_difference.clone();
                margin_right = half_difference.clone();
            }
        }
    }

    layout_box.dimensions.set_width(to_px(width.inner(), containing_block.width));
    layout_box.dimensions.set(
        BoxComponent::Margin,
        Edge::Left,
        to_px(margin_left.inner(), containing_block.width)
    );
    layout_box.dimensions.set(
        BoxComponent::Margin,
        Edge::Right,
        to_px(margin_right.inner(), containing_block.width)
    );
    layout_box.dimensions.set(
        BoxComponent::Padding,
        Edge::Left,
        to_px(padding_left.inner(), containing_block.width)
    );
    layout_box.dimensions.set(
        BoxComponent::Padding,
        Edge::Right,
        to_px(padding_right.inner(), containing_block.width)
    );
    layout_box.dimensions.set(
        BoxComponent::Border,
        Edge::Left,
        to_px(border_left_width.inner(), containing_block.width)
    );
    layout_box.dimensions.set(
        BoxComponent::Border,
        Edge::Right,
        to_px(border_right_width.inner(), containing_block.width)
    );
}
