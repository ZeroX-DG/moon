use crate::box_model::{BoxComponent, Edge};
use crate::formatting_context::{apply_explicit_sizes, layout_children, FormattingContext};
use crate::layout_box::LayoutBox;
use crate::line_box::LineBox;
use style::value_processing::Property;

pub struct InlineFormattingContext {
    line_boxes: Vec<LineBox>,
    containing_block: *mut LayoutBox,
}

impl InlineFormattingContext {
    pub fn new(layout_box: &mut LayoutBox) -> Self {
        Self {
            line_boxes: Vec::new(),
            containing_block: layout_box,
        }
    }

    fn calculate_width(&mut self, layout_box: &mut LayoutBox) {
        let render_node = match &layout_box.render_node {
            Some(node) => node.clone(),
            None => return,
        };

        let containing_block = &self.get_containing_block().dimensions.content;

        let render_node = render_node.borrow();
        let computed_width = render_node.get_style(&Property::Width);
        let computed_margin_left = render_node.get_style(&Property::MarginLeft);
        let computed_margin_right = render_node.get_style(&Property::MarginRight);
        let containing_width = containing_block.width;

        let mut used_width = computed_width.to_px(containing_width);
        let mut used_margin_left = computed_margin_left.to_px(containing_width);
        let mut used_margin_right = computed_margin_right.to_px(containing_width);

        if layout_box.is_non_replaced() && !layout_box.is_inline_block() {
            used_width = 0.0;
            used_margin_left = 0.0;
            used_margin_right = 0.0;
        }

        if layout_box.is_non_replaced() && layout_box.is_inline_block() {
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
        let box_model = layout_box.box_model();
        box_model.set_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
    }

    fn apply_vertical_spacing(&mut self, layout_box: &mut LayoutBox) {
        let containing_block = self.get_containing_block();
        let containing_block = &containing_block.dimensions.content.clone();
        let render_node = layout_box.render_node.clone();
        let box_model = layout_box.box_model();

        if let Some(render_node) = render_node {
            let render_node = render_node.borrow();

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

    fn ensure_last_line_box(&mut self) {
        if self.line_boxes.is_empty() {
            self.line_boxes.push(LineBox::new());
        }
    }
}

impl FormattingContext for InlineFormattingContext {
    fn layout(&mut self, boxes: Vec<&mut LayoutBox>) -> f32 {
        let containing_block = self.get_containing_block();
        let containing_block = &containing_block.dimensions.content.clone();

        self.ensure_last_line_box();

        for layout_box in boxes {
            self.calculate_width(layout_box);
            layout_children(layout_box);
            self.apply_vertical_spacing(layout_box);
            apply_explicit_sizes(layout_box, containing_block);

            let new_width =
                self.line_boxes.last().unwrap().width() + layout_box.dimensions.content.width;

            if new_width > containing_block.width {
                self.line_boxes.push(LineBox::new());
            }

            let line_box = self.line_boxes.last_mut().unwrap();
            line_box.push(layout_box);
        }

        let mut offset_y = 0.;

        for line in &self.line_boxes {
            let mut offset_x = 0.;

            for fragment in line.fragments() {
                let x = containing_block.x + offset_x + fragment.dimensions.margin.left;

                let y = containing_block.y + offset_y + fragment.dimensions.margin.top;

                fragment.box_model().set_position(x, y);
                offset_x += fragment.dimensions.margin_box().width;
            }

            offset_y += line.height();
        }

        offset_y
    }

    fn get_containing_block(&mut self) -> &mut LayoutBox {
        unsafe { self.containing_block.as_mut().unwrap() }
    }
}
