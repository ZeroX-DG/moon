/// This module in charge of the layouting
/// process, which includes:
/// 1. Box width calculation
/// 2. Box position calculation
/// 3. Box height calculation
use super::box_model::{BoxComponent, Edge, Rect};
use super::formatting_context::FormattingContext;
use super::layout_box::LayoutBox;
use style::value_processing::Property;

use super::flow;

pub(crate) struct LayoutContext {
    pub offset_x: f32,
    pub offset_y: f32,
    pub width: f32,
    pub height: f32,
}

/// recursively layout the tree from the root
pub(crate) fn layout(
    root: &mut LayoutBox,
    containing_block: &Rect,
    context: &LayoutContext,
) {
    compute_width(root, containing_block);
    compute_position(root, containing_block, context);
    layout_children(root);
    apply_explicit_sizes(root, containing_block);
}

fn layout_children(root: &mut LayoutBox) {
    let mut context = LayoutContext {
        offset_x: root.dimensions.content.x,
        offset_y: root.dimensions.content.y,
        width: root.dimensions.content.width,
        height: root.dimensions.content.height,
    };

    let containing_block = &root.dimensions.content;

    let formatting_context = root
        .formatting_context
        .clone()
        .expect("No formatting context to layout children");

    for child in root.children.iter_mut() {
        layout(child, containing_block, &context);
        update_context(child, &formatting_context, &mut context);
    }

    if let Some(render_node) = &root.render_node {
        let computed_height = render_node.borrow().get_style(&Property::Height);
        if computed_height.is_auto() || root.is_inline() {
            root.box_model().set_height(context.height);
        }
    }
}

/// Update the layout context to decide where the next box will be rendered.
/// This behavior is depend on different type of layout.
// TODO: Make sure that when we support new layout this will still work, and
// we won't have to touch compute_position function
fn update_context(root: &LayoutBox, parent_context: &FormattingContext, context: &mut LayoutContext) {
    match parent_context {
        FormattingContext::Block => {
            flow::block::update_context(root, context)
        },
        FormattingContext::Inline => {
            flow::inline::update_context(root, context)
        },
        _ => unimplemented!("Unsupported formatting context: {:?}", parent_context),
    }
}

fn compute_position(
    root: &mut LayoutBox,
    containing_block: &Rect,
    context: &LayoutContext,
) {
    let render_node = root.render_node.clone();
    let box_model = root.box_model();

    if let Some(render_node) = render_node {
        let render_node = render_node.borrow();

        let margin_top = render_node
            .get_style(&Property::MarginTop)
            .to_px(containing_block.width);
        let margin_bottom = render_node
            .get_style(&Property::MarginBottom)
            .to_px(containing_block.width);

        let border_top = render_node
            .get_style(&Property::BorderTopWidth)
            .to_px(containing_block.width);
        let border_bottom = render_node
            .get_style(&Property::BorderBottomWidth)
            .to_px(containing_block.width);

        let padding_top = render_node
            .get_style(&Property::PaddingTop)
            .to_px(containing_block.width);
        let padding_bottom = render_node
            .get_style(&Property::PaddingBottom)
            .to_px(containing_block.width);

        box_model.set(BoxComponent::Margin, Edge::Top, margin_top);
        box_model.set(BoxComponent::Margin, Edge::Bottom, margin_bottom);

        box_model.set(BoxComponent::Padding, Edge::Top, padding_top);
        box_model.set(BoxComponent::Padding, Edge::Bottom, padding_bottom);

        box_model.set(BoxComponent::Border, Edge::Top, border_top);
        box_model.set(BoxComponent::Border, Edge::Bottom, border_bottom);
    }

    let content_area_x =
        context.offset_x + box_model.margin.left + box_model.border.left + box_model.padding.left;

    let content_area_y = context.offset_y + box_model.border.top + box_model.padding.top;

    root.box_model()
        .set_position(content_area_x, content_area_y);
}

fn compute_width(root: &mut LayoutBox, containing_block: &Rect) {
    let render_node = match &root.render_node {
        Some(node) => node.clone(),
        None => return
    };
    let is_inline = root.is_inline();
    let is_block = root.is_block();
    let is_float = root.is_float();
    let is_non_replaced = root.is_non_replaced();
    let is_absolutely_positioned = root.is_absolutely_positioned();
    let is_inline_block = root.is_inline_block();
    let is_in_normal_flow = root.is_in_normal_flow();

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
    else if is_block && is_non_replaced && is_in_normal_flow {
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
            computed_margin_right.is_auto(),
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
                used_margin_right = computed_margin_right.to_px(containing_width) + underflow;
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
                    used_margin_right = computed_margin_right.to_px(containing_width) + underflow;
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
    else if is_block && !is_non_replaced && is_in_normal_flow {
        // TODO: work on this when we support replaced elements
    }
    // 5. floating, non-replaced elements
    else if is_float && is_non_replaced {
        // TODO: work on this when we support float elements
    }
    // 6. floating, replaced elements
    else if is_float && !is_non_replaced {
        // TODO: work on this when we support float replaced elements
    }
    // 7. absolutely positioned, non-replaced elements
    else if is_absolutely_positioned && is_non_replaced {
        // TODO: work on this when we support absolutely positioned elements
    }
    // 8. absolutely positioned, replaced elements
    else if is_absolutely_positioned && !is_non_replaced {
        // TODO: work on this when we support absolutely positioned replaced elements
    }
    // 9. 'inline-block', non-replaced elements in normal flow
    else if is_inline_block && is_non_replaced && is_in_normal_flow {
        // TODO: work on this when we support shrink-to-fit
    }
    // 10. 'inline-block', replaced elements in normal flow
    else if is_inline_block && !is_non_replaced && is_in_normal_flow {
        // TODO: work on this when we support inline replaced element -_-
    }

    // apply all calculated used values
    let box_model = root.box_model();
    box_model.set_width(used_width);
    box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
    box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
    box_model.set(
        BoxComponent::Padding,
        Edge::Left,
        computed_padding_left.to_px(containing_width),
    );
    box_model.set(
        BoxComponent::Padding,
        Edge::Right,
        computed_padding_right.to_px(containing_width),
    );
    box_model.set(
        BoxComponent::Border,
        Edge::Left,
        computed_border_left.to_px(containing_width),
    );
    box_model.set(
        BoxComponent::Border,
        Edge::Right,
        computed_border_right.to_px(containing_width),
    );
}

fn apply_explicit_sizes(root: &mut LayoutBox, containing_block: &Rect) {
    if root.is_inline() {
        return
    }

    if let Some(render_node) = &root.render_node {
        let computed_width = render_node.borrow().get_style(&Property::Width);
        let computed_height = render_node.borrow().get_style(&Property::Height);

        if !computed_width.is_auto() {
            let used_width = computed_width.to_px(containing_block.width);
            root.box_model().set_width(used_width);
        }

        if !computed_height.is_auto() {
            let used_height = computed_height.to_px(containing_block.height);
            root.box_model().set_height(used_height);
        }
    }
}

// // #[cfg(test)]
// // mod tests {
// //     use super::*;
// //     use crate::box_gen::build_layout_tree;
// //     use crate::test_utils::print_layout_tree;
// //     use css::cssom::css_rule::CSSRule;
// //     use style::render_tree::build_render_tree;
// //     use style::value_processing::*;
// //     use test_utils::css::parse_stylesheet;
// //     use test_utils::dom_creator::*;

// //     #[test]
// //     fn compute_width_simple() {
// //         let dom = element(
// //             "div",
// //             vec![
// //                 element("span", vec![text("hello")]),
// //                 element("p", vec![text("world")]),
// //                 element("span", vec![text("hello")]),
// //                 element("span", vec![text("hello")]),
// //             ],
// //         );

// //         let css = r#"
// //         div {
// //             display: block;
// //         }
// //         p {
// //             display: block;
// //         }
// //         span {
// //             display: inline;
// //         }
// //         "#;

// //         let stylesheet = parse_stylesheet(css);

// //         let rules = stylesheet
// //             .iter()
// //             .map(|rule| match rule {
// //                 CSSRule::Style(style) => ContextualRule {
// //                     inner: style,
// //                     location: CSSLocation::Embedded,
// //                     origin: CascadeOrigin::User,
// //                 },
// //             })
// //             .collect::<Vec<ContextualRule>>();

// //         let render_tree = build_render_tree(dom.clone(), &rules);
// //         let mut layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

// //         compute_width(
// //             &mut layout_tree,
// //             &mut ContainingBlock {
// //                 rect: Rect {
// //                     x: 0.0,
// //                     y: 0.0,
// //                     width: 1200.0,
// //                     height: 600.0,
// //                 },
// //                 offset_x: 0.0,
// //                 offset_y: 0.0,
// //                 previous_margin_bottom: 0.0,
// //                 collapsed_margins_vertical: 0.0,
// //             },
// //         );

// //         print_layout_tree(&layout_tree, 0);

// //         assert_eq!(layout_tree.dimensions.content.width, 1200.);
// //     }

// //     #[test]
// //     fn layout_simple() {
// //         let dom = element(
// //             "div",
// //             vec![
// //                 element("div", vec![]),
// //                 element("div#blue", vec![]),
// //                 element("div#red", vec![]),
// //             ],
// //         );

// //         let css = r#"
// //         div {
// //             display: block;
// //         }
// //         #blue {
// //             background-color: blue;
// //             height: 340px;
// //             width: 50%;
// //         }
// //         #red {
// //             background-color: red;
// //             height: 200px;
// //             margin-top: 20px;
// //             padding-left: 10px;
// //         }
// //         "#;

// //         let stylesheet = parse_stylesheet(css);

// //         let rules = stylesheet
// //             .iter()
// //             .map(|rule| match rule {
// //                 CSSRule::Style(style) => ContextualRule {
// //                     inner: style,
// //                     location: CSSLocation::Embedded,
// //                     origin: CascadeOrigin::User,
// //                 },
// //             })
// //             .collect::<Vec<ContextualRule>>();

// //         let render_tree = build_render_tree(dom.clone(), &rules);
// //         let mut layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

// //         layout(
// //             &mut layout_tree,
// //             &mut ContainingBlock {
// //                 rect: Rect {
// //                     x: 0.0,
// //                     y: 0.0,
// //                     width: 1200.0,
// //                     height: 600.0,
// //                 },
// //                 offset_x: 0.0,
// //                 offset_y: 0.0,
// //                 previous_margin_bottom: 0.0,
// //                 collapsed_margins_vertical: 0.0,
// //             },
// //         );

// //         print_layout_tree(&layout_tree, 0);

// //         let root_dimensions = &layout_tree.dimensions;
// //         assert_eq!(root_dimensions.content.width, 1200.);
// //         assert_eq!(root_dimensions.content.height, 560.);

// //         let first_child_dimensions = &layout_tree.children[0].dimensions;
// //         assert_eq!(first_child_dimensions.content.width, 1200.);
// //         // since the first div has auto height, its height is 0
// //         assert_eq!(first_child_dimensions.content.height, 0.);

// //         let second_child_dimensions = &layout_tree.children[1].dimensions;
// //         // second div has width of 50%
// //         assert_eq!(second_child_dimensions.content.width, 600.);
// //         assert_eq!(second_child_dimensions.content.height, 340.);

// //         let third_child_dimensions = &layout_tree.children[2].dimensions;
// //         // third div has auto width which decrease to fit the padding
// //         assert_eq!(third_child_dimensions.content.width, 1190.);
// //         assert_eq!(third_child_dimensions.content.height, 200.);
// //         assert_eq!(third_child_dimensions.margin.top, 20.);
// //         assert_eq!(third_child_dimensions.padding.left, 10.);
// //         assert_eq!(third_child_dimensions.content.x, 10.);
// //         assert_eq!(third_child_dimensions.content.y, 360.);
// //     }

// //     #[test]
// //     fn collapse_margin_simple() {
// //         let dom = element(
// //             "div",
// //             vec![
// //                 element("div#yellow", vec![]),
// //                 element("div#blue", vec![]),
// //                 element("div#red", vec![]),
// //             ],
// //         );

// //         let css = r#"
// //         div {
// //             display: block;
// //         }
// //         #yellow {
// //             height: 20px;
// //             margin-bottom: 20px;
// //             background-color: yellow;
// //         }
// //         #blue {
// //             margin-top: 30px;
// //             background-color: blue;
// //             height: 40px;
// //             width: 50%;
// //             margin-bottom: -20px;
// //         }
// //         #red {
// //             background-color: red;
// //             height: 200px;
// //             margin-top: 50px;
// //             padding-left: 10px;
// //         }
// //         "#;

// //         let stylesheet = parse_stylesheet(css);

// //         let rules = stylesheet
// //             .iter()
// //             .map(|rule| match rule {
// //                 CSSRule::Style(style) => ContextualRule {
// //                     inner: style,
// //                     location: CSSLocation::Embedded,
// //                     origin: CascadeOrigin::User,
// //                 },
// //             })
// //             .collect::<Vec<ContextualRule>>();

// //         let render_tree = build_render_tree(dom.clone(), &rules);
// //         let mut layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

// //         layout(
// //             &mut layout_tree,
// //             &mut ContainingBlock {
// //                 rect: Rect {
// //                     x: 0.0,
// //                     y: 0.0,
// //                     width: 1200.0,
// //                     height: 600.0,
// //                 },
// //                 offset_x: 0.0,
// //                 offset_y: 0.0,
// //                 previous_margin_bottom: 0.0,
// //                 collapsed_margins_vertical: 0.0,
// //             },
// //         );

// //         print_layout_tree(&layout_tree, 0);

// //         let root_dimensions = &layout_tree.dimensions;
// //         assert_eq!(root_dimensions.content.width, 1200.);
// //         assert_eq!(root_dimensions.content.height, 320.);

// //         let first_child_dimensions = &layout_tree.children[0].dimensions;
// //         assert_eq!(first_child_dimensions.content.width, 1200.);
// //         assert_eq!(first_child_dimensions.content.height, 20.);
// //         assert_eq!(first_child_dimensions.margin.bottom, 20.);

// //         let second_child_dimensions = &layout_tree.children[1].dimensions;
// //         // second div has width of 50%
// //         assert_eq!(second_child_dimensions.content.width, 600.);
// //         assert_eq!(second_child_dimensions.content.height, 40.);
// //         assert_eq!(second_child_dimensions.margin.top, 30.);
// //         assert_eq!(second_child_dimensions.margin.bottom, -20.);
// //         assert_eq!(second_child_dimensions.content.x, 0.);
// //         assert_eq!(second_child_dimensions.content.y, 50.);

// //         let third_child_dimensions = &layout_tree.children[2].dimensions;
// //         // third div has auto width which decrease to fit the padding
// //         assert_eq!(third_child_dimensions.content.width, 1190.);
// //         assert_eq!(third_child_dimensions.content.height, 200.);
// //         assert_eq!(third_child_dimensions.margin.top, 50.);
// //         assert_eq!(third_child_dimensions.padding.left, 10.);
// //         assert_eq!(third_child_dimensions.content.x, 10.);
// //         assert_eq!(third_child_dimensions.content.y, 120.);
// //     }
// // }
