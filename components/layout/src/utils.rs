use css::cssom::css_rule::CSSRule;
use dom::node::NodePtr;
use style_types::{CSSLocation, CascadeOrigin, ContextualRule};
use test_utils::css::parse_stylesheet;

use crate::layout_box::LayoutBoxPtr;

pub const SHARED_CSS: &str = r#"
html, body {
    display: block;
}
p, div {
    display: block;
}
span, a {
    display: inline;
}
.inline-block {
    display: inline-block;
}
}"#;

pub fn build_tree(dom: NodePtr, css: &str) -> LayoutBoxPtr {
    let document = dom.owner_document().unwrap();
    document.append_child(dom.0.clone());
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

    fn compute_styles(element: NodePtr, style_rules: &[ContextualRule]) {
        let computed_styles = style::compute::compute_styles(element.clone(), &style_rules);
        element.set_computed_styles(computed_styles);

        element.for_each_child(|child| compute_styles(NodePtr(child), style_rules))
    }

    compute_styles(NodePtr(document), &rules);
    crate::tree_builder::TreeBuilder::new().build(dom).unwrap()
}
