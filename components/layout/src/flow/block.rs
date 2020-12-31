use crate::formatting_context::{BaseFormattingContext, FormattingContext};
use crate::layout_box::LayoutBox;
use crate::box_model::{Rect, BoxComponent, Edge};
use style::value_processing::Property;

pub struct BlockFormattingContext {
    base: BaseFormattingContext,
    containing_block: Rect
}

impl BlockFormattingContext {
    pub fn new(rect: &Rect) -> Self {
        Self {
            base: BaseFormattingContext {
                offset_x: rect.x,
                offset_y: rect.y,
                width: 0.,
                height: 0.,
            },
            containing_block: rect.clone()
        }
    }
}

impl FormattingContext for BlockFormattingContext {
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
        let computed_border_left = render_node.get_style(&Property::BorderLeftWidth);
        let computed_border_right = render_node.get_style(&Property::BorderRightWidth);
        let computed_padding_left = render_node.get_style(&Property::PaddingLeft);
        let computed_padding_right = render_node.get_style(&Property::PaddingRight);
        let containing_width = self.containing_block.width;

        let box_width = computed_margin_left.to_px(containing_width)
            + computed_border_left.to_px(containing_width)
            + computed_padding_left.to_px(containing_width)
            + computed_width.to_px(containing_width)
            + computed_padding_right.to_px(containing_width)
            + computed_border_right.to_px(containing_width)
            + computed_margin_right.to_px(containing_width);

        let mut used_width = layout_box.box_model().content.width;
        let mut used_margin_left = layout_box.box_model().margin.left;
        let mut used_margin_right = layout_box.box_model().margin.right;

        // 3. block-level, non-replaced elements in normal flow
        if layout_box.is_non_replaced() {
            if !computed_width.is_auto() && box_width > containing_width {
                if computed_margin_left.is_auto() {
                    used_margin_left = 0.0;
                }
                if computed_margin_right.is_auto() {
                    used_margin_right = 0.0;
                }
            }

            let underflow = containing_width - box_width;

            match (
                computed_width.is_auto(),
                computed_margin_left.is_auto(),
                computed_margin_right.is_auto(),
            ) {
                // If all of the above have a computed value other than 'auto',
                // the values are said to be "over-constrained" and one of the
                // used values will have to be different from its computed value.
                // If the 'direction' property of the containing block has the
                // value 'ltr', the specified value of 'margin-right' is ignored
                // and the value is calculated so as to make the equality true.
                // If the value of 'direction' is 'rtl', this happens to
                // 'margin-left' instead.
                (false, false, false) => {
                    // TODO: support direction rtl
                    used_margin_right = computed_margin_right.to_px(containing_width) + underflow;
                }
                // If there is exactly one value specified as 'auto',
                // its used value follows from the equality.
                (false, true, false) => {
                    used_margin_left = underflow;
                }
                (false, false, true) => {
                    used_margin_right = underflow;
                }
                // If 'width' is set to 'auto', any other 'auto' values become '0'
                // and 'width' follows from the resulting equality.
                (true, _, _) => {
                    if computed_margin_left.is_auto() {
                        used_margin_left = 0.0;
                    }
                    if computed_margin_right.is_auto() {
                        used_margin_right = 0.0;
                    }

                    if underflow >= 0. {
                        used_width = underflow;
                    } else {
                        used_width = 0.;
                        used_margin_right = computed_margin_right.to_px(containing_width) + underflow;
                    }
                }
                // If both 'margin-left' and 'margin-right' are 'auto', their
                // used values are equal. This horizontally centers the element
                // with respect to the edges of the containing block.
                (false, true, true) => {
                    let half_underflow = underflow / 2.;
                    used_margin_left = half_underflow;
                    used_margin_right = half_underflow;
                }
            }
        }

        // apply all calculated used values
        let box_model = layout_box.box_model();
        box_model.set_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
        box_model.set(
            BoxComponent::Padding,
            Edge::Left,
            computed_padding_left.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Padding,
            Edge::Right,
            computed_padding_right.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Border,
            Edge::Left,
            computed_border_left.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Border,
            Edge::Right,
            computed_border_right.to_px(containing_width),
        );
    }

    fn update_new_data(&mut self, layout_box: &LayoutBox) {
        let rect = layout_box.dimensions.margin_box();
        self.base.height += rect.height;
        self.base.offset_y += rect.height;

        if self.base.width < rect.width {
            self.base.width = rect.width;
        }
    }
}