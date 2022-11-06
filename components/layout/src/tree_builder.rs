use dom::node::NodePtr;
use shared::tree_node::TreeNode;
use style_types::{
    values::{display::DisplayBox, prelude::Display},
    Value,
};

use crate::layout_box::{BoxData, LayoutBox, LayoutBoxPtr};

pub struct TreeBuilder {
    parent_stack: Vec<LayoutBoxPtr>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            parent_stack: Vec::new(),
        }
    }

    pub fn build(mut self, root: NodePtr) -> Option<LayoutBoxPtr> {
        let root_node = if root.is_document() {
            // the first child is HTML tag
            root.first_child().map(|n| NodePtr(n))
        } else {
            Some(root)
        };

        if let Some(root_node) = root_node {
            if let Value::Display(Display::Box(DisplayBox::None)) =
                root_node.get_style(&style_types::Property::Display)
            {
                return None;
            }
            let root_box = LayoutBoxPtr(TreeNode::new(LayoutBox::new(root_node.clone())));

            self.parent_stack.push(root_box.clone());
            root_node.for_each_child(|child| {
                self.build_layout_tree(NodePtr(child));
            });
            self.parent_stack.pop();

            return Some(root_box);
        }
        None
    }

    fn build_layout_tree(&mut self, node: NodePtr) {
        if let Value::Display(Display::Box(DisplayBox::None)) =
            node.get_style(&style_types::Property::Display)
        {
            return;
        }
        let layout_box = TreeNode::new(LayoutBox::new(node.clone()));

        let parent = if LayoutBoxPtr(layout_box.clone()).is_inline() {
            self.get_parent_for_inline()
        } else {
            self.get_parent_for_block()
        };

        if parent.is_some() {
            parent.unwrap().append_child(layout_box.clone());

            if !node.is_element() && !node.is_document() {
                return;
            }

            self.parent_stack.push(LayoutBoxPtr(layout_box));
            node.for_each_child(|child| {
                self.build_layout_tree(NodePtr(child));
            });
            self.parent_stack.pop();
        } else {
            let parent_stack = self
                .parent_stack
                .iter()
                .map(|parent| format!("{} {:?}", parent.friendly_name(), parent.node()))
                .collect::<Vec<String>>();
            log::error!(
                "Couldn't find the correct parent for node: {} {:?}",
                LayoutBoxPtr(layout_box).friendly_name(),
                node
            );
            log::error!("Parent stack: {:#?}", parent_stack);
        }
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
    fn get_parent_for_block(&mut self) -> Option<LayoutBoxPtr> {
        let parent = self
            .parent_stack
            .iter()
            .rfind(|parent_box| parent_box.is_block() && parent_box.can_have_children());

        if let Some(parent) = parent {
            if !parent.has_no_child() && parent.children_are_inline() {
                let anonymous = TreeNode::new(LayoutBox::new_anonymous(BoxData::block_box()));

                parent.transfer_children_to_node(anonymous.clone());
                parent.append_child(anonymous);
            }

            return Some(parent.clone());
        }
        None
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
    fn get_parent_for_inline(&mut self) -> Option<LayoutBoxPtr> {
        let parent = self.parent_stack.last();

        if let Some(parent) = parent {
            if parent.children_are_inline() || parent.has_no_child() {
                return Some(parent.clone());
            }

            let require_anonymous_box = match parent.last_child().map(|child| LayoutBoxPtr(child)) {
                Some(last_node) => !(last_node.is_anonymous() && last_node.children_are_inline()),
                None => true,
            };

            if require_anonymous_box {
                let anonymous = TreeNode::new(LayoutBox::new_anonymous(BoxData::block_box()));
                parent.append_child(anonymous);
            }
            return parent.last_child().map(|node| LayoutBoxPtr(node));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{layout_box::LayoutBoxPtr, utils::*};
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

        let root = build_tree(dom, SHARED_CSS);

        // The result box tree should look like this
        // [Block] - Div
        //   |- [Block Anonymous]
        //        |- [Inline] - Span
        //   |- [Block] - P
        //        |- [Inline] - Span
        //        |- [Inline] - Span
        //        |- [Inline] - Span

        assert!(root.is_block());

        assert!(LayoutBoxPtr(root.first_child().unwrap()).is_block());
        assert!(LayoutBoxPtr(root.first_child().unwrap()).is_anonymous());
        assert!(LayoutBoxPtr(root.nth_child(1).unwrap()).is_block());
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

        let root = build_tree(dom, SHARED_CSS);

        // The result box tree should look like this
        // [Block] - Div
        //   |- [Block Anonymous]
        //        |- [Inline] - Span
        //   |- [Block] - P
        //   |- [Block Anonymous]
        //        |- [Inline] - A
        //        |- [Inline] - A
        //        |- [Inline] - A

        assert!(root.is_block());

        assert_eq!(root.children_count(), 3);

        assert!(LayoutBoxPtr(root.first_child().unwrap()).is_block());
        assert!(LayoutBoxPtr(root.first_child().unwrap()).is_anonymous());

        assert!(LayoutBoxPtr(root.nth_child(1).unwrap()).is_block());

        assert!(LayoutBoxPtr(root.nth_child(2).unwrap()).is_block());
        assert!(LayoutBoxPtr(root.nth_child(2).unwrap()).is_anonymous());
    }
}
