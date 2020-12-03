/// This module is responsible for the box generation
/// of elements in the render tree. In other words,
/// this module transforms render tree to layout tree
/// to prepare for layouting process.
use super::layout_box::{BoxType, FormattingContext, LayoutBox, LayoutTree, LayoutBoxNode};
use super::{
    is_block_container_box, is_block_level_element, is_inline_level_element, is_text_node,
};
use style::render_tree::RenderNodeRef;

/// Build the layout tree from root render node
///
/// This will generate boxes for each render node & construct
/// a layout tree for the layouting process
pub fn build_layout_tree(root: RenderNodeRef, layout_tree: &mut LayoutTree) -> Option<&LayoutBoxNode> {
    if let Some(root_box_type) = get_box_type(&root) {
        let mut root_box = layout_tree.new_node_instance(LayoutBox::new(root.clone(), root_box_type));

        let children = root
            .borrow()
            .children
            .iter()
            .filter_map(|child| build_layout_tree(child.clone(), layout_tree))
            .collect::<Vec<&LayoutBoxNode>>();

        let has_block = children
            .iter()
            .find(|child| match child.box_type {
                BoxType::Block => true,
                _ => false,
            })
            .is_some();

        match has_block {
            true => root_box.set_formatting_context(FormattingContext::Block),
            false if is_block_container_box(&root_box) => {
                root_box.set_formatting_context(FormattingContext::Inline);
            }
            _ => { /* This one has no formatting context. It's just a box */ }
        }

        for child in children {
            layout_tree.add_child(root_box, *child);
        }

        return Some(root_box);
    }
    println!("Don't know which box type for this: {:#?}", root);
    None
}

/// Get a box type for a node
pub fn get_box_type(root: &RenderNodeRef) -> Option<BoxType> {
    if is_text_node(&root) {
        Some(BoxType::Anonymous)
    } else if is_block_level_element(&root) {
        Some(BoxType::Block)
    } else if is_inline_level_element(&root) {
        Some(BoxType::Inline)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout_box::*;
    use crate::test_utils::print_layout_tree;
    use css::cssom::css_rule::CSSRule;
    use style::render_tree::build_render_tree;
    use style::value_processing::*;
    use test_utils::css::parse_stylesheet;
    use test_utils::dom_creator::*;
    use test_utils::printing::print_dom_tree;

    #[test]
    fn test_build_tree() {
        let dom = element(
            "div",
            vec![
                element("span", vec![text("hello")]),
                element("p", vec![text("world")]),
                element("span", vec![text("hello")]),
                element("span", vec![text("hello")]),
            ],
        );

        print_dom_tree(dom.clone(), 0);

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
        let mut layout_tree = LayoutTree::new();

        let root = build_layout_tree(render_tree.root.unwrap(), &mut layout_tree).unwrap();

        println!("------------------------");
        print_layout_tree(&root, 0);

        assert_eq!(root.box_type, BoxType::Block);
        assert_eq!(
            root.formatting_context,
            Some(FormattingContext::Block)
        );
        // span
        let span = layout_tree.get_node_direct(&root.children[0]);
        assert_eq!(span.box_type, BoxType::Anonymous);
        assert_eq!(
            span.formatting_context,
            Some(FormattingContext::Inline)
        );
        assert_eq!(
            layout_tree.get_node_direct(&span.children[0]).box_type,
            BoxType::Inline
        );
        // p
        let p = layout_tree.get_node_direct(&root.children[1]);
        assert_eq!(p.box_type, BoxType::Block);
        assert_eq!(
            p.formatting_context,
            Some(FormattingContext::Inline)
        );
        assert_eq!(
            layout_tree.get_node_direct(&p.children[0]).box_type,
            BoxType::Anonymous
        );
        // last 2 span is grouped
        let grouped = layout_tree.get_node_direct(&p.children[2]);
        assert_eq!(grouped.box_type, BoxType::Anonymous);
        assert_eq!(
            grouped.formatting_context,
            Some(FormattingContext::Inline)
        );
        assert_eq!(
            layout_tree.get_node_direct(&grouped.children[0]).box_type,
            BoxType::Inline
        );
        assert_eq!(
            layout_tree.get_node_direct(&grouped.children[1]).box_type,
            BoxType::Inline
        );
    }
}
