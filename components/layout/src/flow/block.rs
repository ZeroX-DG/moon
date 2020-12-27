use crate::layout::LayoutContext;
use crate::layout_box::LayoutBox;

/// Update layout context for each box in block formatting context.
/// The only thing will be changed is the parent box height & the offset y
/// to put the next box at. Since every box in BFC has full width & layout
/// from top to bottom, we don't need to update width or offsett x.
pub(crate) fn update_context(root: &LayoutBox, context: &mut LayoutContext) {
    let child_margin_height = root.dimensions.margin_box_height();
    context.height += child_margin_height;
    context.offset_y += child_margin_height - root.dimensions.margin.bottom;
}