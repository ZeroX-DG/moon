use style::{
    property::Property,
    render_tree::RenderNodeRef,
    value::Value,
    values::display::{Display, InnerDisplayType, OuterDisplayType},
};

use crate::{
    flow::{block::BlockBox, inline::InlineBox},
    layout_box::{children_are_inline, LayoutBox, LayoutNodeId, LayoutTree},
};

pub struct TreeBuilder {
    parent_stack: Vec<LayoutNodeId>,
    tree: LayoutTree,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            parent_stack: Vec::new(),
            tree: LayoutTree::new(),
        }
    }

    pub fn build(mut self, root: RenderNodeRef) -> LayoutTree {
        let root_box = match self.build_box_by_display(&root) {
            Some(b) => b,
            None => return self.tree,
        };

        let root_box_id = self.tree.set_root(root_box);

        self.parent_stack.push(root_box_id);
        for child in &root.borrow().children {
            self.build_layout_tree(child.clone());
        }
        self.parent_stack.pop();

        self.tree
    }

    fn build_layout_tree(&mut self, node: RenderNodeRef) {
        let layout_box = match self.build_box_by_display(&node) {
            Some(b) => b,
            None => return,
        };

        let parent = if layout_box.is_inline() {
            self.get_parent_for_inline()
        } else {
            self.get_parent_for_block()
        };

        self.tree.add_child(&parent, layout_box);

        let box_ref = self.tree.children(&parent).last().unwrap();

        self.parent_stack.push(*box_ref);
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
    fn get_parent_for_block(&mut self) -> LayoutNodeId {
        while let Some(parent_box) = self.parent_stack.last() {
            let current_box = self.tree.get_node(parent_box);
            if current_box.is_inline() {
                self.parent_stack.pop();
            } else {
                break;
            }
        }

        if self.parent_stack.last().is_none() {
            panic!("Can't find block parent for block box");
        }

        let parent = self.parent_stack.last().unwrap();

        if children_are_inline(&self.tree, parent) {
            let children = self.tree.children_mut(parent).drain(..).collect::<Vec<_>>();
            let anonymous = Box::new(BlockBox::new_anonymous());
            let anonymous_box_id = self.tree.add_child(parent, anonymous);

            self.tree
                .get_node_mut(&anonymous_box_id)
                .set_children(children);
        }

        *parent
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
    fn get_parent_for_inline(&mut self) -> LayoutNodeId {
        let parent = self.parent_stack.last().expect("No parent in stack");

        if children_are_inline(&self.tree, parent) {
            return *parent;
        }

        if let Some(last) = self.tree.children(parent).last() {
            let last_node = self.tree.get_node(last);
            if !last_node.is_anonymous() || !children_are_inline(&self.tree, &last) {
                let anonymous = Box::new(BlockBox::new_anonymous());
                self.tree.add_child(parent, anonymous);
            }
        } else {
            let anonymous = Box::new(BlockBox::new_anonymous());
            self.tree.add_child(parent, anonymous);
        }

        *self.tree.children(parent).last().unwrap()
    }

    fn build_box_by_display(&self, node: &RenderNodeRef) -> Option<Box<dyn LayoutBox>> {
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

        Some(layout_box)
    }
}

#[cfg(test)]
mod tests {
    use crate::{layout_printer::dump_layout, utils::*};
    use test_utils::dom_creator::*;

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

        let layout_tree = build_tree(dom, SHARED_CSS);
        let root = layout_tree.root().unwrap();

        // The result box tree should look like this
        // [Block] - Div
        //   |- [Block Anonymous]
        //        |- [Inline] - Span
        //   |- [Block] - P
        //        |- [Inline] - Span
        //        |- [Inline] - Span
        //        |- [Inline] - Span

        assert!(layout_tree.get_node(&root).is_block());

        assert!(layout_tree
            .get_node(&layout_tree.children(&root)[0])
            .is_block());
        assert!(layout_tree
            .get_node(&layout_tree.children(&root)[0])
            .is_anonymous());
        assert!(layout_tree
            .get_node(&layout_tree.children(&root)[1])
            .is_block());
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

        let layout_tree = build_tree(dom, SHARED_CSS);
        let root = layout_tree.root().unwrap();

        // The result box tree should look like this
        // [Block] - Div
        //   |- [Block Anonymous]
        //        |- [Inline] - Span
        //   |- [Block] - P
        //   |- [Block Anonymous]
        //        |- [Inline] - A
        //        |- [Inline] - A
        //        |- [Inline] - A

        dump_layout(&layout_tree, &root);
        assert!(layout_tree.get_node(&root).is_block());

        assert_eq!(layout_tree.children(&root).len(), 3);

        assert!(layout_tree
            .get_node(&layout_tree.children(&root)[0])
            .is_block());
        assert!(layout_tree
            .get_node(&layout_tree.children(&root)[0])
            .is_anonymous());

        assert!(layout_tree
            .get_node(&layout_tree.children(&root)[1])
            .is_block());

        assert!(layout_tree
            .get_node(&layout_tree.children(&root)[2])
            .is_block());
        assert!(layout_tree
            .get_node(&layout_tree.children(&root)[2])
            .is_anonymous());
    }
}
