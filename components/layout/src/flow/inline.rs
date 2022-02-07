use std::rc::Rc;

use crate::{box_model::BoxComponent, formatting_context::LayoutContext, layout_box::LayoutBox};
use dom::node::NodeData;
use shared::primitive::edge::Edge;
use style::property::Property;

use super::line_box::{LineBox, LineBoxBuilder, LineFragment};

pub struct InlineFormattingContext {
    layout_context: Rc<LayoutContext>,
    line_boxes: Vec<LineBox>,
}

impl InlineFormattingContext {
    pub fn new(layout_context: Rc<LayoutContext>) -> Self {
        Self {
            layout_context,
            line_boxes: Vec::new(),
        }
    }

    pub fn run(&mut self, layout_node: Rc<LayoutBox>) {
        self.generate_line_boxes(layout_node.clone());

        let containing_block = layout_node.dimensions().content_box();

        let mut offset_y = 0.;

        for line in &self.line_boxes {
            let mut offset_x = 0.;

            for fragment in line.fragments() {
                match fragment {
                    LineFragment::Box(box_fragment) => {
                        let x = containing_block.x + offset_x + box_fragment.dimensions().margin.left;
                        let y = containing_block.y + offset_y + box_fragment.dimensions().margin.top;

                        box_fragment.dimensions_mut().set_position(x, y);
                        offset_x += box_fragment.dimensions().margin_box().width;
                    }
                }
            }

            offset_y += line.height();
        }

        layout_node.dimensions_mut().set_height(offset_y);
    }

    fn generate_line_boxes(&mut self, layout_node: Rc<LayoutBox>) {
        let mut line_box_builder = LineBoxBuilder::new(layout_node.clone());
        self.line_boxes.clear();

        for child in layout_node.children().iter() {
            match child.render_node() {
                Some(render_node) => match render_node.node.data() {
                    Some(NodeData::Text(_)) | Some(NodeData::Comment(_)) => {}
                    _ => {
                        self.layout_dimension_box(child.clone());
                        line_box_builder.add_box_fragment(child.clone());
                    }
                }
                _ => {
                    self.layout_dimension_box(child.clone());
                    line_box_builder.add_box_fragment(child.clone());
                }
            }
        }
        self.line_boxes = line_box_builder.finish();
    }

    fn layout_dimension_box(&mut self, layout_node: Rc<LayoutBox>) {
        self.calculate_width_for_element(layout_node.clone());

        layout_node
            .formatting_context()
            .run(self.layout_context.clone(), layout_node.clone());

        self.apply_vertical_spacing(layout_node.clone());
        layout_node.apply_explicit_sizes();
    }

    fn calculate_width_for_element(&mut self, layout_node: Rc<LayoutBox>) {
        let containing_block = layout_node.containing_block().dimensions().content_box();

        let render_node = match layout_node.render_node() {
            Some(node) => node.clone(),
            _ => return,
        };

        let computed_width = render_node.get_style(&Property::Width);
        let computed_margin_left = render_node.get_style(&Property::MarginLeft);
        let computed_margin_right = render_node.get_style(&Property::MarginRight);
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
        let mut box_model = layout_node.dimensions_mut();
        box_model.set_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
    }

    fn apply_vertical_spacing(&mut self, layout_node: Rc<LayoutBox>) {
        let containing_block = layout_node.containing_block().dimensions().content_box();

        let render_node = layout_node.render_node();
        let mut box_model = layout_node.dimensions_mut();

        if let Some(render_node) = render_node {
            let margin_top = render_node
                .get_style(&Property::MarginTop)
                .to_px(containing_block.width);
            let margin_bottom = render_node
                .get_style(&Property::MarginBottom)
                .to_px(containing_block.width);

            let border_top = render_node
                .get_style(&Property::BorderTopWidth)
                .to_px(containing_block.width);
            let border_bottom = render_node
                .get_style(&Property::BorderBottomWidth)
                .to_px(containing_block.width);

            let padding_top = render_node
                .get_style(&Property::PaddingTop)
                .to_px(containing_block.width);
            let padding_bottom = render_node
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
