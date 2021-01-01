use crate::box_model::{BoxComponent, Edge, Rect};
use crate::formatting_context::{BaseFormattingContext, FormattingContext};
use crate::layout_box::LayoutBox;
use style::value_processing::Property;

pub struct InlineFormattingContext {
    base: BaseFormattingContext,
    containing_block: Rect,
}

impl InlineFormattingContext {
    pub fn new(rect: &Rect) -> Self {
        Self {
            base: BaseFormattingContext {
                offset_x: rect.x,
                offset_y: rect.y,
                width: 0.,
                height: 0.,
            },
            containing_block: rect.clone(),
        }
    }
}

impl FormattingContext for InlineFormattingContext {
    fn base(&self) -> &BaseFormattingContext {
        &self.base
    }

    fn calculate_width(&mut self, layout_box: &mut LayoutBox) {
        let render_node = match &layout_box.render_node {
            Some(node) => node.clone(),
            None => return,
        };

        let render_node = render_node.borrow();
        let computed_width = render_node.get_style(&Property::Width);
        let computed_margin_left = render_node.get_style(&Property::MarginLeft);
        let computed_margin_right = render_node.get_style(&Property::MarginRight);
        let containing_width = self.containing_block.width;

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
                
            }
        }

        // apply all calculated used values
        let box_model = layout_box.box_model();
        box_model.set_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
    }

    fn update_new_data(&mut self, layout_box: &LayoutBox) {
        let rect = layout_box.dimensions.margin_box();
        self.base.width += rect.width;
        self.base.offset_x += rect.width;

        if self.base.height < rect.height {
            self.base.height = rect.height;
        }
    }
}
