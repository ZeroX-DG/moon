pub mod layout_box;
pub mod box_model;
pub mod box_generation;

use box_generation::{generate_box, wrap_inline_boxes};
use style::render_tree::RenderNodeRef;
use layout_box::LayoutBox;

/// Build the layout tree from root render node
///
/// There are 3 steps to this process
/// 1. **Box generation:** Generate boxes base on render tree
///     1. **Generate tree:** generate layout boxes for elements
///     2. **Wrap inline boxes:** when block-level box breaks the inline boxes 
///     this process wrap those inline boxes into anonymous block box
/// 2. **Size calculation:** Calculate the size of all generated boxes
/// 3. **Position calculation:** Calculate the position of all generated boxes
pub fn build_layout_tree(root: RenderNodeRef) -> Option<LayoutBox> {
    let mut root_box = generate_box(root.clone());
    if let Some(b) = &mut root_box {
        wrap_inline_boxes(b);
    }
    root_box
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::layout_box::*;
    use test_utils::css::parse_stylesheet;
    use test_utils::dom_creator::*;
    use css::cssom::css_rule::CSSRule;
    use style::value_processing::*;
    use style::render_tree::build_render_tree;
    use dom::dom_ref::NodeRef;

    fn print_tree(root: NodeRef, level: usize) {
        let child_nodes = root.borrow().as_node().child_nodes();
        println!(
            "{}{:#?}({} child)",
            "    ".repeat(level),
            root,
            child_nodes.length()
        );
        for node in child_nodes {
            print_tree(node, level + 1);
        }
    }

    #[test]
    fn test_build_tree() {
        let dom = element("div", vec![
            element("span", vec![text("hello")]),
            element("p", vec![text("world")]),
            element("span", vec![text("hello")]),
            element("span", vec![text("hello")]),
        ]);

        print_tree(dom.clone(), 0);

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
        let layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

        assert_eq!(layout_tree.box_type, BoxType::Block);
        assert_eq!(layout_tree.fmt_context, Some(FormattingContext::Block));
        // span
        assert_eq!(layout_tree.children[0].box_type, BoxType::Anonymous);
        assert_eq!(layout_tree.children[0].fmt_context, Some(FormattingContext::Inline));
        assert_eq!(layout_tree.children[0].children[0].box_type, BoxType::Inline);
        // p
        assert_eq!(layout_tree.children[1].box_type, BoxType::Block);
        assert_eq!(layout_tree.children[1].fmt_context, Some(FormattingContext::Inline));
        assert_eq!(layout_tree.children[1].children[0].box_type, BoxType::Anonymous);
        // last 2 span is grouped
        assert_eq!(layout_tree.children[2].box_type, BoxType::Anonymous);
        assert_eq!(layout_tree.children[2].fmt_context, Some(FormattingContext::Inline));
        assert_eq!(layout_tree.children[2].children[0].box_type, BoxType::Inline);
        assert_eq!(layout_tree.children[2].children[1].box_type, BoxType::Inline);
    }
}
