use crate::box_model::{BoxComponent, Edge};
use crate::formatting_context::{FormattingContext, apply_explicit_sizes, layout_children};
use crate::layout_box::LayoutBox;
use style::value_processing::Property;

#[derive(Debug)]
struct BaseFormattingContext {
    pub offset_x: f32,
    pub width: f32,
    pub height: f32,
}

pub struct InlineFormattingContext {
    base: BaseFormattingContext,
    containing_block: *mut LayoutBox,
}

impl InlineFormattingContext {
    pub fn new(layout_box: &mut LayoutBox) -> Self {
        let rect = &layout_box.dimensions.content;

        Self {
            base: BaseFormattingContext {
                offset_x: rect.x,
                width: 0.,
                height: 0.,
            },
            containing_block: layout_box,
        }
    }

    fn update_new_data(&mut self, layout_box: &LayoutBox) {
        let rect = layout_box.dimensions.margin_box();
        self.base.width += rect.width;
        self.base.offset_x += rect.width;
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

    fn calculate_position(&mut self, layout_box: &mut LayoutBox) {
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

        let content_area_x =
            self.base.offset_x + box_model.margin.left + box_model.border.left + box_model.padding.left;

        let content_area_y =
            containing_block.y + box_model.margin.top + box_model.border.top + box_model.padding.top;

        layout_box
            .box_model()
            .set_position(content_area_x, content_area_y);
    }
}

impl FormattingContext for InlineFormattingContext {
    fn layout(&mut self, boxes: Vec<&mut LayoutBox>) -> f32 {
        let containing_block = self.get_containing_block();
        let containing_block = &containing_block.dimensions.content.clone();

        for layout_box in boxes {
            self.calculate_width(layout_box);
            self.calculate_position(layout_box);
            layout_children(layout_box);
            apply_explicit_sizes(layout_box, containing_block);
            self.update_new_data(layout_box);
        }

        self.base.height
    }

    fn get_containing_block(&mut self) -> &mut LayoutBox {
        unsafe {self.containing_block.as_mut().unwrap()}
    }
}
