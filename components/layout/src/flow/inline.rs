use crate::formatting_context::{BaseFormattingContext, FormattingContext};
use crate::layout_box::LayoutBox;
use crate::box_model::{Rect, BoxComponent, Edge};

pub struct InlineFormattingContext {
    base: BaseFormattingContext
}

impl InlineFormattingContext {
    pub fn new(rect: &Rect) -> Self {
        Self {
            base: BaseFormattingContext {
                offset_x: rect.x,
                offset_y: rect.y,
                width: 0.,
                height: 0.,
            }
        }
    }
}

impl FormattingContext for InlineFormattingContext {
    fn base(&self) -> &BaseFormattingContext {
        &self.base
    }

    fn calculate_width(&mut self, layout_box: &mut LayoutBox) {
        let mut used_width = layout_box.box_model().content.width;
        let mut used_margin_left = layout_box.box_model().margin.left;
        let mut used_margin_right = layout_box.box_model().margin.right;

        if layout_box.is_non_replaced() {
            used_width = 0.0;
            used_margin_left = 0.0;
            used_margin_right = 0.0;
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