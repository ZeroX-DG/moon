/// This module in charge of the layouting
/// process, which includes:
/// 1. Box width calculation
/// 2. Box position calculation
/// 3. Box height calculation
use super::layout_box::LayoutBox;
use super::{
    is_inline_level_element,
    is_non_replaced_element,
    is_block_level_element,
    is_float_element,
    is_absolutely_positioned,
    is_inline_block
};
use super::box_model::{BoxComponent, Edge};
use style::value_processing::Property;

pub struct ContainingBlock {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

/// recursively layout the tree from the root
pub fn layout(root: &mut LayoutBox, containing_block: &ContainingBlock) {
    compute_width(root, containing_block);
}

pub fn compute_width(root: &mut LayoutBox, containing_block: &ContainingBlock) {
    let render_node = root.render_node.clone();
    let is_inline = is_inline_level_element(&render_node);
    let is_block = is_block_level_element(&render_node);
    let is_float = is_float_element(&render_node);
    let is_non_replaced = is_non_replaced_element(&render_node);
    let is_absolutely_positioned = is_absolutely_positioned(&render_node);
    let is_inline_block = is_inline_block(&render_node);

    let render_node = render_node.borrow();
    let computed_width = render_node.get_style(&Property::Width);
    let computed_margin_left = render_node.get_style(&Property::MarginLeft);
    let computed_margin_right = render_node.get_style(&Property::MarginRight);
    let computed_border_left = render_node.get_style(&Property::BorderLeftWidth);
    let computed_border_right = render_node.get_style(&Property::BorderRightWidth);
    let computed_padding_left = render_node.get_style(&Property::PaddingLeft);
    let computed_padding_right = render_node.get_style(&Property::PaddingRight);
    let containing_width = containing_block.width;

    let box_width = computed_margin_left.to_px(containing_width)
        + computed_border_left.to_px(containing_width)
        + computed_padding_left.to_px(containing_width)
        + computed_width.to_px(containing_width)
        + computed_padding_right.to_px(containing_width)
        + computed_border_right.to_px(containing_width)
        + computed_margin_right.to_px(containing_width);

    let mut used_width = root.box_model().content.width;
    let mut used_margin_left = root.box_model().margin.left;
    let mut used_margin_right = root.box_model().margin.right;

    // 1. inline, non-replaced elements
    if is_inline && is_non_replaced {
        used_width = 0.0;
        used_margin_left = 0.0;
        used_margin_right = 0.0;
    }

    // 2. inline, replaced elements
    else if is_inline && !is_non_replaced {
        // TODO: work on this when we support replaced elements
    }

    // 3. block-level, non-replaced elements in normal flow
    else if is_block && is_non_replaced {
        if !computed_width.is_auto() && box_width > containing_width {
            if computed_margin_left.is_auto() {
                used_margin_left = 0.0;
            }
            if computed_margin_right.is_auto() {
                used_margin_right = 0.0;
            }
        }

        let underflow = containing_width - box_width;

        match (
            computed_width.is_auto(),
            computed_margin_left.is_auto(),
            computed_margin_right.is_auto()
        ) {
            // If all of the above have a computed value other than 'auto', 
            // the values are said to be "over-constrained" and one of the
            // used values will have to be different from its computed value.
            // If the 'direction' property of the containing block has the 
            // value 'ltr', the specified value of 'margin-right' is ignored 
            // and the value is calculated so as to make the equality true.
            // If the value of 'direction' is 'rtl', this happens to 
            // 'margin-left' instead.
            (false, false, false) => {
                // TODO: support direction rtl
                used_margin_right =
                    computed_margin_right.to_px(containing_width) + underflow;
            }
            // If there is exactly one value specified as 'auto', 
            // its used value follows from the equality.
            (false, true, false) => {
                used_margin_left = underflow;
            }
            (false, false, true) => {
                used_margin_right = underflow;
            }
            // If 'width' is set to 'auto', any other 'auto' values become '0'
            // and 'width' follows from the resulting equality. 
            (true, _, _) => {
                if computed_margin_left.is_auto() {
                    used_margin_left = 0.0;
                }
                if computed_margin_right.is_auto() {
                    used_margin_right = 0.0;
                }

                if underflow >= 0. {
                    used_width = underflow;
                } else {
                    used_width = 0.;
                    used_margin_right =
                        computed_margin_right.to_px(containing_width) + underflow;
                }
            }
            // If both 'margin-left' and 'margin-right' are 'auto', their
            // used values are equal. This horizontally centers the element 
            // with respect to the edges of the containing block.
            (false, true, true) => {
                let half_underflow = underflow / 2.;
                used_margin_left = half_underflow;
                used_margin_right = half_underflow;
            }
        }
    }
    
    // 4. block-level, replaced elements in normal flow
    else if is_block && !is_non_replaced {
        // TODO: work on this when we support replaced elements
    }

    else if is_float && is_non_replaced {
        // TODO: work on this when we support float elements
    }

    else if is_float && !is_non_replaced {
        // TODO: work on this when we support float replaced elements
    }

    else if is_absolutely_positioned && is_non_replaced {
        // TODO: work on this when we support absolutely positioned elements
    }

    else if is_absolutely_positioned && !is_non_replaced {
        // TODO: work on this when we support absolutely positioned replaced elements
    }

    else if is_inline_block && is_non_replaced {
        // TODO: work on this when we support shrink-to-fit
    }

    else if is_inline_block && !is_non_replaced {
        // TODO: work on this when we support inline replaced element -_-
    }

    // apply all calculated used values
    root.box_model().set_width(used_width);
    root.box_model().set(BoxComponent::Margin, Edge::Left, used_margin_left);
    root.box_model().set(BoxComponent::Margin, Edge::Right, used_margin_right);
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::dom_creator::*;
    use css::cssom::css_rule::CSSRule;
    use style::render_tree::build_render_tree;
    use style::value_processing::*;
    use test_utils::css::parse_stylesheet;
    use crate::box_gen::build_layout_tree;
    use crate::test_utils::print_layout_tree;

    #[test]
    fn compute_width_simple() {
        let dom = element(
            "div",
            vec![
                element("span", vec![text("hello")]),
                element("p", vec![text("world")]),
                element("span", vec![text("hello")]),
                element("span", vec![text("hello")]),
            ],
        );

        let css = r#"
        div {
            display: block;
        }
        p {
            display: block;
        }
        span {
            display: inline;
        }
        "#;

        let stylesheet = parse_stylesheet(css);

        let rules = stylesheet
            .iter()
            .map(|rule| match rule {
                CSSRule::Style(style) => ContextualRule {
                    inner: style,
                    location: CSSLocation::Embedded,
                    origin: CascadeOrigin::User,
                },
            })
            .collect::<Vec<ContextualRule>>();

        let render_tree = build_render_tree(dom.clone(), &rules);
        let mut layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

        compute_width(&mut layout_tree, &ContainingBlock {
            x: 0.0,
            y: 0.0,
            width: 1200.0,
            height: 600.0
        });

        print_layout_tree(&layout_tree, 0);
    }
}
