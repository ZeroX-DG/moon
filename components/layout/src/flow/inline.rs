use crate::{
    box_model::BoxComponent,
    formatting_context::{BaseFormattingContext, FormattingContext, LayoutContext},
    layout_box::LayoutBoxPtr,
};
use dom::node::NodeData;
use regex::Regex;
use shared::primitive::edge::Edge;
use style_types::Property;

use super::line_box::LineBoxBuilder;

pub struct InlineBoxIterator {
    stack: Vec<LayoutBoxPtr>,
}

impl InlineBoxIterator {
    pub fn new(parent: LayoutBoxPtr) -> Self {
        Self {
            stack: parent
                .iterate_children()
                .rev()
                .map(|child| LayoutBoxPtr(child))
                .collect(),
        }
    }
}

impl Iterator for InlineBoxIterator {
    type Item = LayoutBoxPtr;
    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }
        let ptr = self.stack.pop().unwrap();

        for child in ptr.iterate_children().rev() {
            self.stack.push(LayoutBoxPtr(child));
        }

        return Some(ptr);
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
                            return;
                        }
                        // TODO: Support different line break types
                        let regex = Regex::new(r"\s|\t|\n").unwrap();
                        for word in regex.split(text_content.trim()) {
                            if word.is_empty() {
                                continue;
                            }
                            line_box_builder.add_text_fragment(child.clone(), word.to_string());
                            line_box_builder.add_text_fragment(child.clone(), ' '.to_string());
                        }
                    }
                    Some(NodeData::Element(_)) => {
                        self.layout_dimension_box(context, child.clone());
                        line_box_builder.add_box_fragment(child.clone());
                    }
                    _ => {}
                },
                _ => {
                    self.layout_dimension_box(context, child.clone());
                    line_box_builder.add_box_fragment(child.clone());
                }
            }
        }
        *layout_node.lines().borrow_mut() = line_box_builder.finish();
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
