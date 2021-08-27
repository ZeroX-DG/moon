use style::{
    render_tree::RenderNodeRef,
    value_processing::{Property, Value},
    values::display::{Display, InnerDisplayType, OuterDisplayType},
};

use crate::{
    flow::BlockBox,
    layout_box::{LayoutBox, LayoutNode, LayoutNodePtr},
};

pub struct TreeBuilder {
    parent_stack: Vec<LayoutNodePtr>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            parent_stack: Vec::new()
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
            // TODO: Support inline node
            return;
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

        return LayoutNodePtr::from(parent_mut);
    }

    fn build_box_by_display(&self, node: &RenderNodeRef) -> Option<LayoutNode> {
        // TODO: support text
        if node.borrow().node.is_text() {
            return None;
        }

        let display = node.borrow().get_style(&Property::Display);

        let layout_box: Box<dyn LayoutBox> = Box::new(match display.inner() {
            Value::Display(d) => match d {
                Display::Full(outer, inner) => match (outer, inner) {
                    (OuterDisplayType::Block, InnerDisplayType::Flow) => BlockBox::new(),
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
        });

        let layout_node = LayoutNode::new_boxed(layout_box);

        Some(layout_node)
    }
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

        let mut layout_tree_builder = TreeBuilder::new();

        let layout_box = layout_tree_builder.build(render_tree.root.unwrap());

        let layout_box = layout_box.unwrap();

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
}
