/// This module is responsible for the box generation
/// of elements in the render tree. In other words,
/// this module transforms render tree to layout tree
/// to prepare for layouting process.
use super::layout_box::{BoxType, LayoutBox};
use std::cell::RefCell;
use std::rc::Rc;
use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::{Display, InnerDisplayType, OuterDisplayType};

pub struct TreeBuilder {
    parent_stack: Rc<RefCell<Vec<*mut LayoutBox>>>,
    root: RenderNodeRef,
}

impl TreeBuilder {
    pub fn new(root: RenderNodeRef) -> Self {
        Self {
            parent_stack: Rc::new(RefCell::new(Vec::new())),
            root,
        }
    }

    /// Build the layout tree for the provided root render node
    pub fn build(mut self) -> Option<LayoutBox> {
        let root = self.root.clone();
        let mut root_box = match build_box_by_display(&root) {
            Some(b) => b,
            None => return None,
        };

        self.parent_stack.borrow_mut().push(&mut root_box);
        for child in &root.borrow().children {
            self.build_layout_tree(child.clone());
        }
        self.parent_stack.borrow_mut().pop();

        return Some(root_box);
    }

    /// Recursively building the layout tree for a node
    fn build_layout_tree(&mut self, node: RenderNodeRef) -> Option<&LayoutBox> {
        let layout_box = match build_box_by_display(&node) {
            Some(b) => b,
            None => return None,
        };

        let parent = unsafe {
            if layout_box.is_inline() {
                get_parent_for_inline(self.parent_stack.clone())
            } else {
                get_parent_for_block(self.parent_stack.clone())
            }
        };

        parent.add_child(layout_box);

        let box_ref = parent.children.last_mut().unwrap();

        self.parent_stack.borrow_mut().push(box_ref);
        for child in &node.borrow().children {
            self.build_layout_tree(child.clone());
        }
        self.parent_stack.borrow_mut().pop();

        parent.children.last()
    }
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
unsafe fn get_parent_for_inline<'a>(
    parent_stack: Rc<RefCell<Vec<*mut LayoutBox>>>,
) -> &'a mut LayoutBox {
    let parent_stack = parent_stack.borrow();
    let parent = parent_stack.last().expect("No parent in stack");

    let parent_mut = parent
        .as_mut()
        .expect("Can't get mutable reference to parent");

    if parent_mut.children_are_inline() {
        return parent_mut;
    }

    if let Some(last) = parent_mut.children.last() {
        if !last.is_anonymous() || !last.children_are_inline() {
            let mut anonymous = LayoutBox::new_anonymous(BoxType::Block);
            anonymous.set_children_inline(true);
            parent_mut.add_child(anonymous);
        }
    } else {
        let mut anonymous = LayoutBox::new_anonymous(BoxType::Block);
        anonymous.set_children_inline(true);
        parent_mut.add_child(anonymous);
    }

    parent_mut.children.last_mut().unwrap()
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
unsafe fn get_parent_for_block<'a>(
    parent_stack: Rc<RefCell<Vec<*mut LayoutBox>>>,
) -> &'a mut LayoutBox {
    let mut parent_stack = parent_stack.borrow_mut();

    while let Some(parent_box) = parent_stack.last() {
        let current_box = parent_box.as_ref().unwrap();
        if !current_box.is_inline_block() && current_box.is_inline() {
            parent_stack.pop();
        } else {
            break;
        }
    }

    if parent_stack.last().is_none() {
        panic!("Can't find block parent for block box");
    }

    let parent_mut = parent_stack
        .last()
        .unwrap()
        .as_mut()
        .expect("Can't get mutable reference to parent");

    if parent_mut.children_are_inline() {
        let children = parent_mut.children.drain(..).collect::<Vec<_>>();
        let mut anonymous = LayoutBox::new_anonymous(BoxType::Block);
        anonymous.children = children;
        anonymous.set_children_inline(true);
        parent_mut.add_child(anonymous);
        parent_mut.set_children_inline(false);
    }

    return parent_mut;
}

fn all_inline_children(node: &RenderNodeRef) -> bool {
    for child in &node.borrow().children {
        match child.borrow().get_style(&Property::Display).inner() {
            Value::Display(Display::Full(OuterDisplayType::Block, _)) => return false,
            _ => {}
        }
    }
    true
}

fn build_box_by_display(node: &RenderNodeRef) -> Option<LayoutBox> {
    // TODO: support text
    if node.borrow().node.is_text() {
        return None;
    }

    let display = node.borrow().get_style(&Property::Display);

    let box_type = match display.inner() {
        Value::Display(d) => match d {
            Display::Full(outer, inner) => match (outer, inner) {
                (OuterDisplayType::Block, InnerDisplayType::Flow) => BoxType::Block,
                (OuterDisplayType::Inline, InnerDisplayType::Flow)
                | (OuterDisplayType::Inline, InnerDisplayType::FlowRoot) => BoxType::Inline,
                _ => return None,
            },
            _ => {
                log::warn!("Unsupport display type: {:#?}", d);
                return None;
            }
        },
        _ => unreachable!(),
    };

    let mut layout_box = LayoutBox::new(node.clone(), box_type.clone());

    if all_inline_children(node) {
        layout_box.set_children_inline(true);
    }

    if !layout_box.is_inline_block() {
        if let BoxType::Inline = layout_box.box_type {
            layout_box.set_children_inline(true);
        }
    }

    Some(layout_box)
}

#[cfg(test)]
mod tests {
    use super::*;
    use css::cssom::css_rule::CSSRule;
    use style::build_render_tree;
    use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};
    use test_utils::css::parse_stylesheet;
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

        let css = r#"
        p, div {
            display: block;
        }
        span {
            display: inline;
        }"#;

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

        let layout_tree_builder = TreeBuilder::new(render_tree.root.unwrap());

        let layout_box = layout_tree_builder.build();

        let layout_box = layout_box.unwrap();

        // The result box tree should look like this
        // [Block] - Div
        //   |- [Block Anonymous]
        //        |- [Inline] - Span
        //   |- [Block] - P
        //        |- [Inline] - Span
        //        |- [Inline] - Span
        //        |- [Inline] - Span

        assert!(layout_box.box_type == BoxType::Block);

        assert!(layout_box.children[0].box_type == BoxType::Block);
        assert!(layout_box.children[0].is_anonymous());

        assert!(layout_box.children[1].box_type == BoxType::Block);
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

        let css = r#"
        p, div {
            display: block;
        }
        span, a {
            display: inline;
        }"#;

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

        let layout_tree_builder = TreeBuilder::new(render_tree.root.unwrap());

        let layout_box = layout_tree_builder.build();

        let layout_box = layout_box.unwrap();

        // The result box tree should look like this
        // [Block] - Div
        //   |- [Block Anonymous]
        //        |- [Inline] - Span
        //   |- [Block] - P
        //   |- [Block Anonymous]
        //        |- [Inline] - A
        //        |- [Inline] - A
        //        |- [Inline] - A

        assert!(layout_box.box_type == BoxType::Block);

        assert_eq!(layout_box.children.len(), 3);

        assert!(layout_box.children[0].box_type == BoxType::Block);
        assert!(layout_box.children[0].is_anonymous());

        assert!(layout_box.children[1].box_type == BoxType::Block);
        assert!(!layout_box.children[1].is_anonymous());

        assert!(layout_box.children[2].box_type == BoxType::Block);
        assert!(layout_box.children[2].is_anonymous());
    }
}
