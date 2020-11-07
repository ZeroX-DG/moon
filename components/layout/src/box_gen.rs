/// This module is responsible for the box generation
/// of elements in the render tree. In other words,
/// this module transforms render tree to layout tree
/// to prepare for layouting process.
use super::layout_box::{LayoutBox, BoxType, FormattingContext};
use super::{
    is_block_container_box,
    is_inline_level_element,
    is_block_level_element,
    is_text_node,
};
use style::render_tree::RenderNodeRef;

/// Build the layout tree from root render node
/// 
/// This will generate boxes for each render node & construct
/// a layout tree for the layouting process
pub fn build_layout_tree(root: RenderNodeRef) -> Option<LayoutBox> {
    if let Some(root_box_type) = get_box_type(&root) {
        let mut root_box = LayoutBox::new(root.clone(), root_box_type);

        let children = root.borrow().children
            .iter()
            .filter_map(|child| build_layout_tree(child.clone()))
            .collect::<Vec<LayoutBox>>();

        let has_block = children
            .iter()
            .find(|child| match child.box_type {
                BoxType::Block => true,
                _ => false
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
            root_box.add_child(child);
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
    use crate::layout_box::*;
    use super::*;
    use css::cssom::css_rule::CSSRule;
    use dom::dom_ref::NodeRef;
    use style::render_tree::build_render_tree;
    use style::value_processing::*;
    use test_utils::css::parse_stylesheet;
    use test_utils::dom_creator::*;

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

    fn print_layout(root: &LayoutBox, level: usize) {
        let child_nodes = &root.children;
        println!(
            "{}{:#?}({:#?})({} child)",
            "    ".repeat(level),
            root.box_type,
            root.render_node.borrow().node,
            child_nodes.len()
        );
        for node in child_nodes {
            print_layout(node, level + 1);
        }
    }

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

        println!("------------------------");
        print_layout(&layout_tree, 0);

        assert_eq!(layout_tree.box_type, BoxType::Block);
        assert_eq!(layout_tree.formatting_context, Some(FormattingContext::Block));
        // span
        assert_eq!(layout_tree.children[0].box_type, BoxType::Anonymous);
        assert_eq!(
            layout_tree.children[0].formatting_context,
            Some(FormattingContext::Inline)
        );
        assert_eq!(
            layout_tree.children[0].children[0].box_type,
            BoxType::Inline
        );
        // p
        assert_eq!(layout_tree.children[1].box_type, BoxType::Block);
        assert_eq!(
            layout_tree.children[1].formatting_context,
            Some(FormattingContext::Inline)
        );
        assert_eq!(
            layout_tree.children[1].children[0].box_type,
            BoxType::Anonymous
        );
        // last 2 span is grouped
        assert_eq!(layout_tree.children[2].box_type, BoxType::Anonymous);
        assert_eq!(
            layout_tree.children[2].formatting_context,
            Some(FormattingContext::Inline)
        );
        assert_eq!(
            layout_tree.children[2].children[0].box_type,
            BoxType::Inline
        );
        assert_eq!(
            layout_tree.children[2].children[1].box_type,
            BoxType::Inline
        );
    }
}
