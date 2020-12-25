/// This module is responsible for the box generation
/// of elements in the render tree. In other words,
/// this module transforms render tree to layout tree
/// to prepare for layouting process.
use super::layout_box::{LayoutBox, BoxType, FormattingContext};
use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::{Display, OuterDisplayType, InnerDisplayType};

pub struct TreeBuilder {
    parent_stack: Vec<*mut LayoutBox>,
    root: RenderNodeRef
}

impl TreeBuilder {
    pub fn new(root: RenderNodeRef) -> Self {
        Self {
            parent_stack: Vec::new(),
            root
        }
    }

    pub fn build(mut self) -> Option<LayoutBox> {
        let root = self.root.clone();
        let mut root_box = match build_box_by_display(&root) {
            Some(b) => b,
            None => return None
        };

        for child in &root.borrow().children {
            self.parent_stack.push(&mut root_box);
            self.build_layout_tree(child.clone());
            self.parent_stack.pop();
        }

        return Some(root_box);
    }

    pub fn build_layout_tree(&mut self, node: RenderNodeRef) -> Option<&LayoutBox> {
        let mut layout_box = match build_box_by_display(&node) {
            Some(b) => b,
            None => return None
        };

        for child in &node.borrow().children {
            self.parent_stack.push(&mut layout_box);
            self.build_layout_tree(child.clone());
            self.parent_stack.pop();
        }

        unsafe {
            let parent = self.parent_stack
                .last()
                .expect("No parent in stack")
                .as_mut()
                .expect("Can't get mutable reference to parent");
            parent.add_child(layout_box);

            parent.children.last()
        }
    }
}

fn establish_inline_context(node: &RenderNodeRef) -> bool {
    for child in &node.borrow().children {
        match child.borrow().get_style(&Property::Display).inner() {
            Value::Display(
                Display::Full(OuterDisplayType::Block, _)
            ) => return false,
            _ => {}
        }
    }
    true
}

fn build_box_by_display(node: &RenderNodeRef) -> Option<LayoutBox> {
    let display = node.borrow().get_style(&Property::Display);

    let (box_type, formatting_context) = match display.inner() {
        Value::Display(d) => match d {
            Display::Full(outer, inner) => match (outer, inner) {
                (OuterDisplayType::Block, InnerDisplayType::Flow) => {
                    let formatting_context = if establish_inline_context(node) {
                        FormattingContext::Inline
                    } else {
                        FormattingContext::Block
                    };
                    (BoxType::Block, formatting_context)
                }
                (OuterDisplayType::Inline, InnerDisplayType::Flow) => {
                    (BoxType::Inline, FormattingContext::Inline)
                }
                _ => return None
            }
            _ => {
                log::warn!("Unsupport display type: {:#?}", d);
                return None;
            }
        }
        _ => unreachable!()
    };

    let mut layout_box = LayoutBox::new(node.clone(), box_type);
    layout_box.set_formatting_context(formatting_context);

    Some(layout_box)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::dom_creator::*;
    use test_utils::css::parse_stylesheet;
    use css::cssom::css_rule::CSSRule;
    use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};
    use style::build_render_tree;

    #[test]
    fn test_build_simple() {
        let dom = element("div#parent", vec![
            element("span", vec![
            ])
        ]);

        let css = r#"div {
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

        println!("{:#?}", layout_box.unwrap().to_string());
    }
}
