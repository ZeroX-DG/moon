use std::rc::Rc;

use crate::{
    box_model::BoxComponent,
    formatting_context::{BaseFormattingContext, FormattingContext},
    layout_box::LayoutBoxPtr, layout_context::LayoutContext,
};
use dom::node::NodeData;
use regex::Regex;
use shared::primitive::edge::Edge;
use style_types::Property;

use super::line_box::LineBoxBuilder;

pub struct InlineBoxIterator {
    stack: Vec<LayoutBoxPtr>,
    visited: Vec<LayoutBoxPtr>,
}

impl InlineBoxIterator {
    pub fn new(parent: LayoutBoxPtr) -> Self {
        Self {
            stack: vec![parent.clone()],
            visited: vec![parent],
        }
    }
}

impl Iterator for InlineBoxIterator {
    type Item = LayoutBoxPtr;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.last() {
            let mut maybe_found_node = None;
            // if top of the stack is a leaf then remove it and skip to the next element
            if node.has_no_child() {
                self.stack.pop();
                return self.next();
            }

            for child in node.iterate_children() {
                if !self.visited.iter().any(|n| Rc::ptr_eq(&n.0, &child)) {
                    let child_box = LayoutBoxPtr(child.clone());
                    self.stack.push(child_box.clone());
                    self.visited.push(child_box.clone());
                    maybe_found_node = Some(child_box);
                    break;
                }
            }

            if let Some(found_node) = maybe_found_node {
                return Some(found_node);
            } else {
                self.stack.pop();
                return self.next();
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct InlineFormattingContext {
    base: BaseFormattingContext,
}

impl FormattingContext for InlineFormattingContext {
    fn run(&self, context: &LayoutContext, layout_node: LayoutBoxPtr) {
        if !layout_node.is_block() {
            log::debug!("Attempt to run IFC on non-block box");
            return;
        }

        self.generate_line_boxes(context, layout_node.clone());

        let content_height: f32 = layout_node
            .lines()
            .borrow()
            .iter()
            .map(|line| line.size.height)
            .sum();

        layout_node.set_content_height(content_height);
    }

    fn base(&self) -> &BaseFormattingContext {
        &self.base
    }
}

impl InlineFormattingContext {
    pub fn new(base: BaseFormattingContext) -> Self {
        Self { base }
    }

    fn generate_line_boxes(&self, context: &LayoutContext, layout_node: LayoutBoxPtr) {
        let mut line_box_builder = LineBoxBuilder::new(layout_node.clone());
        layout_node.lines().borrow_mut().clear();

        let inline_child_iter = InlineBoxIterator::new(layout_node.clone());

        for child in inline_child_iter {
            match child.node() {
                Some(node) => match node.data() {
                    Some(NodeData::Text(content)) => {
                        let text_content = content.get_data();
                        if text_content.trim().is_empty() {
                            continue;
                        }
                        // TODO: Support different line break types
                        let regex = Regex::new(r"\s|\t|\n").unwrap();
                        for word in regex.split(text_content.trim()) {
                            if word.is_empty() {
                                continue;
                            }
                            line_box_builder.add_text_fragment(context, child.clone(), word.to_string());
                            line_box_builder.add_text_fragment(context, child.clone(), ' '.to_string());
                        }
                    }
                    Some(NodeData::Element(_)) => {
                        self.layout_dimension_box(context, child.clone());
                        line_box_builder.add_box_fragment(context, child.clone());
                    }
                    _ => {}
                },
                _ => {
                    self.layout_dimension_box(context, child.clone());
                    line_box_builder.add_box_fragment(context, child.clone());
                }
            }
        }
        *layout_node.lines().borrow_mut() = line_box_builder.finish(context);
    }

    fn layout_dimension_box(&self, context: &LayoutContext, layout_node: LayoutBoxPtr) {
        self.calculate_width_for_element(layout_node.clone());

        self.layout_inside(context, layout_node.clone());

        self.apply_vertical_spacing(layout_node.clone());
        layout_node.apply_explicit_sizes();
    }

    fn calculate_width_for_element(&self, layout_node: LayoutBoxPtr) {
        let containing_block = layout_node.containing_block().unwrap().content_size();

        let node = match layout_node.node() {
            Some(node) => node.clone(),
            _ => return,
        };

        let computed_width = node.get_style(&Property::Width);
        let computed_margin_left = node.get_style(&Property::MarginLeft);
        let computed_margin_right = node.get_style(&Property::MarginRight);
        let containing_width = containing_block.width;

        let mut used_width = computed_width.to_px(containing_width);
        let mut used_margin_left = computed_margin_left.to_px(containing_width);
        let mut used_margin_right = computed_margin_right.to_px(containing_width);

        if layout_node.is_non_replaced() && !layout_node.is_inline_block() {
            used_width = 0.0;
            used_margin_left = 0.0;
            used_margin_right = 0.0;
        }

        if layout_node.is_non_replaced() && layout_node.is_inline_block() {
            if computed_margin_left.is_auto() {
                used_margin_left = 0.0;
            }
            if computed_margin_right.is_auto() {
                used_margin_right = 0.0;
            }
            if computed_width.is_auto() {
                // TODO: Support auto width when we have shrink-to-fit width
            }
        }

        // apply all calculated used values
        let mut box_model = layout_node.box_model.borrow_mut();
        layout_node.set_content_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
    }

    fn apply_vertical_spacing(&self, layout_node: LayoutBoxPtr) {
        let containing_block = layout_node.containing_block().unwrap().content_size();

        let node = layout_node.node();
        let mut box_model = layout_node.box_model.borrow_mut();

        if let Some(node) = node {
            let margin_top = node
                .get_style(&Property::MarginTop)
                .to_px(containing_block.width);
            let margin_bottom = node
                .get_style(&Property::MarginBottom)
                .to_px(containing_block.width);

            let border_top = node
                .get_style(&Property::BorderTopWidth)
                .to_px(containing_block.width);
            let border_bottom = node
                .get_style(&Property::BorderBottomWidth)
                .to_px(containing_block.width);

            let padding_top = node
                .get_style(&Property::PaddingTop)
                .to_px(containing_block.width);
            let padding_bottom = node
                .get_style(&Property::PaddingBottom)
                .to_px(containing_block.width);

            box_model.set(BoxComponent::Margin, Edge::Top, margin_top);
            box_model.set(BoxComponent::Margin, Edge::Bottom, margin_bottom);

            box_model.set(BoxComponent::Padding, Edge::Top, padding_top);
            box_model.set(BoxComponent::Padding, Edge::Bottom, padding_bottom);

            box_model.set(BoxComponent::Border, Edge::Top, border_top);
            box_model.set(BoxComponent::Border, Edge::Bottom, border_bottom);
        }
    }
}

#[cfg(test)]
mod tests {
    use shared::primitive::{Rect, Size};
    use test_utils::dom_creator::{document, element};

    use crate::{
        formatting_context::{establish_context, FormattingContextType},
        layout_box::LayoutBoxPtr,
        utils::{build_tree, SHARED_CSS}, layout_context::LayoutContext,
    };

    use super::InlineBoxIterator;

    #[test]
    fn test_iterate_inline() {
        let document = document();
        let dom = element(
            "div",
            document.clone(),
            vec![
                element("a", document.clone(), vec![]),
                element("a", document.clone(), vec![]),
                element("a", document.clone(), vec![]),
                element(
                    "div.inline-block#a",
                    document.clone(),
                    vec![
                        element("div.inline-block#b", document.clone(), vec![]),
                        element("div.inline-block#c", document.clone(), vec![]),
                        element("div.inline-block#d", document.clone(), vec![]),
                    ],
                ),
                element(
                    "div.inline-block#f",
                    document.clone(),
                    vec![
                        element(
                            "div.inline-block#g",
                            document.clone(),
                            vec![element("div.inline-block#z", document.clone(), vec![])],
                        ),
                        element("div.inline-block#h", document.clone(), vec![]),
                        element("div.inline-block#j", document.clone(), vec![]),
                    ],
                ),
            ],
        );

        let root = build_tree(dom, SHARED_CSS);
        let mut inline_iterator = InlineBoxIterator::new(root);

        let mapper1 = |e: LayoutBoxPtr| e.node().unwrap().as_element().tag_name();
        let mapper2 = |e: LayoutBoxPtr| {
            let node = e.node().unwrap();
            let element = node.as_element();
            (element.tag_name(), element.id().unwrap())
        };

        assert_eq!(inline_iterator.next().map(mapper1), Some("a".to_string()));
        assert_eq!(inline_iterator.next().map(mapper1), Some("a".to_string()));
        assert_eq!(inline_iterator.next().map(mapper1), Some("a".to_string()));
        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "a".to_string()))
        );
        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "b".to_string()))
        );
        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "c".to_string()))
        );
        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "d".to_string()))
        );

        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "f".to_string()))
        );
        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "g".to_string()))
        );
        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "z".to_string()))
        );
        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "h".to_string()))
        );
        assert_eq!(
            inline_iterator.next().map(mapper2),
            Some(("div".to_string(), "j".to_string()))
        );

        assert_eq!(inline_iterator.next().map(mapper1), None);
    }

    #[test]
    fn test_inline_format() {
        let document = document();
        let dom = element(
            "div",
            document.clone(),
            vec![
                element("div.inline-block", document.clone(), vec![]),
                element("div.inline-block", document.clone(), vec![]),
                element("div.inline-block", document.clone(), vec![]),
            ],
        );

        let root = build_tree(dom, SHARED_CSS);

        let layout_context = LayoutContext {
            viewport: Rect::new(0., 0., 500., 300.),
            measure_text_fn: Box::new(|_, _| Size::new(0., 0.))
        };

        establish_context(FormattingContextType::InlineFormattingContext, root.clone());
        root.formatting_context().run(&layout_context, root.clone());

        assert_eq!(root.lines().borrow().len(), 1);
        assert_eq!(
            root.lines().borrow().first().map(|l| l.fragments.len()),
            Some(3)
        );
    }
}
