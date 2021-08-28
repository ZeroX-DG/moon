use css::cssom::css_rule::CSSRule;
use dom::dom_ref::NodeRef;
use style::{build_render_tree, value_processing::{CSSLocation, CascadeOrigin, ContextualRule}};
use test_utils::css::parse_stylesheet;

use crate::{layout_box::LayoutNode, tree_builder::TreeBuilder};

pub const SHARED_CSS: &str = r#"
p, div {
    display: block;
}
span, a {
    display: inline;
}"#;

pub fn build_tree(dom: NodeRef, css: &str) -> LayoutNode {
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

    let mut layout_tree_builder = TreeBuilder::new();

    let layout_box = layout_tree_builder.build(render_tree.root.unwrap());

    layout_box.unwrap()
}