use style::{
    render_tree::RenderNodeRef,
    value_processing::{Property, Value},
    values::display::{Display, InnerDisplayType, OuterDisplayType},
};

use crate::{
    flow::{block::BlockBox, inline::InlineBox},
    layout_box::{LayoutBox, LayoutNode, LayoutNodePtr},
};

pub struct TreeBuilder {
    parent_stack: Vec<LayoutNodePtr>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            parent_stack: Vec::new(),
        }
    }

    pub fn build(&mut self, root: RenderNodeRef) -> Option<LayoutNode> {
        let mut root_box = match self.build_box_by_display(&root) {
            Some(b) => b,
            None => return None,
        };

        self.parent_stack.push(LayoutNodePtr::from(&mut root_box));
        for child in &root.borrow().children {
            self.build_layout_tree(child.clone());
        }
        self.parent_stack.pop();

        return Some(root_box);
    }

    fn build_layout_tree(&mut self, node: RenderNodeRef) {
        let layout_box = match self.build_box_by_display(&node) {
            Some(b) => b,
            None => return,
        };

        let mut parent = if layout_box.is_inline() {
            self.get_parent_for_inline()
        } else {
            self.get_parent_for_block()
        };

        let parent = parent.as_mut();

        parent.add_child(layout_box);

        let box_ref = parent.children_mut().last_mut().unwrap();

        self.parent_stack.push(LayoutNodePtr::from(box_ref));
        for child in &node.borrow().children {
            self.build_layout_tree(child.clone());
        }
        self.parent_stack.pop();
    }

    /// Get a parent for an block-level box
    ///
    /// A block-level box can only be inserted into the nearest non-inline parent.
    ///
    /// If the parent established a non-inline formatting context, then
    /// insert the box as a direct children of the parent.
    ///
    /// Otherwise, if the nearest parent established an inline formatting
    /// context, then create an anonymous block-level box to wrap all the
    /// inline-level boxes currently in the parent. After that, set the
    /// formatting context of parent to block and insert the box as a direct
    /// children of the parent.
    fn get_parent_for_block(&mut self) -> LayoutNodePtr {
        while let Some(parent_box) = self.parent_stack.last() {
            let current_box = parent_box.as_ref();
            if current_box.is_inline() {
                self.parent_stack.pop();
            } else {
                break;
            }
        }

        if self.parent_stack.last().is_none() {
            panic!("Can't find block parent for block box");
        }

        let parent_mut = self.parent_stack.last_mut().unwrap().as_mut();

        if parent_mut.children_are_inline() {
            let children = parent_mut.children_mut().drain(..).collect::<Vec<_>>();
            let mut anonymous = LayoutNode::new(BlockBox::new_anonymous());
            anonymous.set_children(children);
            parent_mut.add_child(anonymous);
        }

        LayoutNodePtr::from(parent_mut)
    }

    /// Get a parent for an inline-level box
    ///
    /// An inline-level box can be inserted into the nearest parent.
    ///
    /// If the nearest parent established an inline formatting context, then
    /// insert the box as a direct children of the parent.
    ///
    /// Otherwise, if the nearest parent established a block formatting context
    /// then create an anonymous block-level box to wrap the inline-box in before
    /// inserting into the parent.
    fn get_parent_for_inline(&mut self) -> LayoutNodePtr {
        let parent = self.parent_stack.last_mut().expect("No parent in stack");

        let parent_mut = parent.as_mut();

        if parent_mut.children_are_inline() {
            return LayoutNodePtr::from(parent_mut);
        }

        if let Some(last) = parent_mut.children().last() {
            if !last.is_anonymous() || !last.children_are_inline() {
                let anonymous = LayoutNode::new(BlockBox::new_anonymous());
                parent_mut.add_child(anonymous);
            }
        } else {
            let anonymous = LayoutNode::new(BlockBox::new_anonymous());
            parent_mut.add_child(anonymous);
        }

        LayoutNodePtr::from(parent_mut.children_mut().last_mut().unwrap())
    }

    fn build_box_by_display(&self, node: &RenderNodeRef) -> Option<LayoutNode> {
        // TODO: support text
        if node.borrow().node.is_text() {
            return None;
        }

        let display = node.borrow().get_style(&Property::Display);

        let layout_box: Box<dyn LayoutBox> = match display.inner() {
            Value::Display(d) => match d {
                Display::Full(outer, inner) => match (outer, inner) {
                    (OuterDisplayType::Block, InnerDisplayType::Flow) => {
                        Box::new(BlockBox::new(node.clone()))
                    }
                    (OuterDisplayType::Inline, InnerDisplayType::Flow)
                    | (OuterDisplayType::Inline, InnerDisplayType::FlowRoot) => {
                        Box::new(InlineBox::new(node.clone()))
                    }
                    _ => {
                        log::warn!("Unsupport display type: {:#?}", d);
                        return None;
                    }
                },
                _ => {
                    log::warn!("Unsupport display type: {:#?}", d);
                    return None;
                }
            },
            _ => unreachable!(),
        };

        let layout_node = LayoutNode::new_boxed(layout_box);

        Some(layout_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css::cssom::css_rule::CSSRule;
    use dom::dom_ref::NodeRef;
    use style::build_render_tree;
    use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};
    use test_utils::css::parse_stylesheet;
    use test_utils::dom_creator::*;

    const SHARED_CSS: &str = r#"
    p, div {
        display: block;
    }
    span, a {
        display: inline;
    }"#;

    fn build_tree(dom: NodeRef, css: &str) -> LayoutNode {
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

    #[test]
    fn test_build_simple() {
        let document = document();
        let dom = element(
            "div",
            document.clone(),
            vec![
                element("span", document.clone(), vec![]),
                element(
                    "p",
                    document.clone(),
                    vec![
                        element("span", document.clone(), vec![]),
                        element("span", document.clone(), vec![]),
                        element("span", document.clone(), vec![]),
                    ],
                ),
            ],
        );

        let layout_box = build_tree(dom, SHARED_CSS);

        // The result box tree should look like this
        // [Block] - Div
        //   |- [Block Anonymous]
        //        |- [Inline] - Span
        //   |- [Block] - P
        //        |- [Inline] - Span
        //        |- [Inline] - Span
        //        |- [Inline] - Span

        assert!(layout_box.is_block());

        assert!(layout_box.children()[0].as_ref().is_block());
        assert!(layout_box.children()[0].as_ref().is_anonymous());

        assert!(layout_box.children()[1].as_ref().is_block());
    }

    #[test]
    fn test_block_break_inline() {
        let document = document();
        let dom = element(
            "div",
            document.clone(),
            vec![
                element("span", document.clone(), vec![]),
                element("p", document.clone(), vec![]),
                element("a", document.clone(), vec![]),
                element("a", document.clone(), vec![]),
                element("a", document.clone(), vec![]),
            ],
        );

        let layout_box = build_tree(dom, SHARED_CSS);

        // The result box tree should look like this
        // [Block] - Div
        //   |- [Block Anonymous]
        //        |- [Inline] - Span
        //   |- [Block] - P
        //   |- [Block Anonymous]
        //        |- [Inline] - A
        //        |- [Inline] - A
        //        |- [Inline] - A

        assert!(layout_box.is_block());

        assert_eq!(layout_box.children().len(), 3);

        assert!(layout_box.children()[0].is_block());
        assert!(layout_box.children()[0].is_anonymous());

        assert!(layout_box.children()[1].is_block());
        assert!(!layout_box.children()[1].is_anonymous());

        assert!(layout_box.children()[2].is_block());
        assert!(layout_box.children()[2].is_anonymous());
    }
}
