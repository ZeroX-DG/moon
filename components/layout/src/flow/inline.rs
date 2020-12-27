use crate::layout::LayoutContext;
use crate::layout_box::LayoutBox;

/// Update layout context for each box in inline formatting context.
pub(crate) fn update_context(root: &LayoutBox, context: &mut LayoutContext) {
    let child_margin_width = root.dimensions.margin_box_width();
    context.width += child_margin_width;
    context.offset_x += child_margin_width - root.dimensions.margin.right;
}