use crate::{
    formatting_context::{establish_context, establish_context_for, FormattingContextType},
    layout_box::{BoxData, LayoutBox},
};
use std::rc::Rc;
use style::render_tree::RenderNode;

pub struct TreeBuilder {
    parent_stack: Vec<Rc<LayoutBox>>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            parent_stack: Vec::new(),
        }
    }

    pub fn build(mut self, root: Rc<RenderNode>) -> Rc<LayoutBox> {
        let root_box = Rc::new(LayoutBox::new(root.clone()));

        self.parent_stack.push(root_box.clone());
        for child in root.children.borrow().iter() {
            self.build_layout_tree(child.clone());
        }
        self.parent_stack.pop();
        establish_context_for(root_box.clone());

        root_box
    }

    fn build_layout_tree(&mut self, node: Rc<RenderNode>) {
        let layout_box = Rc::new(LayoutBox::new(node.clone()));

        let parent = if layout_box.is_inline() {
            self.get_parent_for_inline()
        } else {
            self.get_parent_for_block()
        };

        LayoutBox::add_child(parent.clone(), layout_box.clone());

        let box_ref = parent.children().last().unwrap().clone();

        self.parent_stack.push(box_ref);
        for child in node.children.borrow().iter() {
            self.build_layout_tree(child.clone());
        }
        self.parent_stack.pop();
        establish_context_for(layout_box.clone());
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
    fn get_parent_for_block(&mut self) -> Rc<LayoutBox> {
        while let Some(parent_box) = self.parent_stack.last() {
            if parent_box.is_inline() {
                self.parent_stack.pop();
            } else {
                break;
            }
        }

        if self.parent_stack.last().is_none() {
            panic!("Can't find block parent for block box");
        }

        let parent = self.parent_stack.last().unwrap().clone();

        if parent.children_are_inline() {
            let children = parent.children_mut().drain(..).collect::<Vec<_>>();
            let anonymous = Rc::new(LayoutBox::new_anonymous(BoxData::block_box()));
            establish_context(
                FormattingContextType::BlockFormattingContext,
                anonymous.clone(),
            );

            LayoutBox::set_children(anonymous.clone(), children);
            LayoutBox::add_child(parent.clone(), anonymous);
        }

        parent
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
    fn get_parent_for_inline(&mut self) -> Rc<LayoutBox> {
        let parent = self
            .parent_stack
            .last()
            .expect("No parent in stack")
            .clone();

        if parent.children_are_inline() {
            return parent;
        }

        let require_anonymous_box = match parent.children().last() {
            Some(last_node) => !last_node.is_anonymous() || !last_node.children_are_inline(),
            None => true,
        };

        if require_anonymous_box {
            let anonymous = Rc::new(LayoutBox::new_anonymous(BoxData::block_box()));
            establish_context(
                FormattingContextType::InlineFormattingContext,
                anonymous.clone(),
            );
            LayoutBox::add_child(parent.clone(), anonymous);
        }

        let children = parent.children();
        children.last().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::*;
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

        assert!(root.children()[0].is_block());
        assert!(root.children()[0].is_anonymous());
        assert!(root.children()[1].is_block());
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

        assert_eq!(root.children().len(), 3);

        assert!(root.children()[0].is_block());
        assert!(root.children()[0].is_anonymous());

        assert!(root.children()[1].is_block());

        assert!(root.children()[2].is_block());
        assert!(root.children()[2].is_anonymous());
    }
}
