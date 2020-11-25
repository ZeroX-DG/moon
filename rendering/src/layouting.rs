use dom::dom_ref::NodeRef;
use css::cssom::stylesheet::StyleSheet;
use css::cssom::css_rule::CSSRule;
use style::value_processing::{ContextualRule, CSSLocation, CascadeOrigin};
use style::render_tree::build_render_tree;
use layout::{build_layout_tree, ContainingBlock, layout_box::LayoutBox};

pub fn layout(
    dom: &NodeRef,
    stylesheets: &[StyleSheet],
    width: f32,
    height: f32
) -> LayoutBox {
    let style_rules = stylesheets.iter().fold(vec![], |mut acc, stylesheet| {
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
        acc.extend(rules);
        acc
    });

    let render_tree = build_render_tree(dom.clone(), &style_rules);
    let mut layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

    layout::layout(&mut layout_tree, &mut ContainingBlock {
        offset_x: 0.,
        offset_y: 0.,
        x: 0.,
        y: 0.,
        width,
        height,
        previous_margin_bottom: 0.0,
        collapsed_margins_vertical: 0.0
    });

    log::debug!("Done layout");

    layout_tree
}
