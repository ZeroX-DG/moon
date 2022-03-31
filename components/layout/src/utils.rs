use std::rc::Rc;

use css::cssom::css_rule::CSSRule;
use dom::node::Node;
use style_types::{CSSLocation, CascadeOrigin, ContextualRule};
use test_utils::css::parse_stylesheet;

use crate::layout_box::LayoutBox;

pub const SHARED_CSS: &str = r#"
p, div {
    display: block;
}
span, a {
    display: inline;
}"#;

pub fn build_tree(dom: Rc<Node>, css: &str) -> Rc<LayoutBox> {
    let stylesheet = parse_stylesheet(css);

    let rules = stylesheet
        .iter()
        .map(|rule| match rule {
            CSSRule::Style(style) => ContextualRule {
                inner: style.clone(),
                location: CSSLocation::Embedded,
                origin: CascadeOrigin::User,
            },
        })
        .collect::<Vec<ContextualRule>>();

    let render_tree = style::tree_builder::TreeBuilder::build(dom.clone(), &rules);
    crate::tree_builder::TreeBuilder::new().build(render_tree.root.unwrap())
}
