use crate::layout::LayoutContext;
use crate::layout_box::LayoutBox;

/// Update layout context for each box in block formatting context.
/// The only thing will be changed is the parent box height & the offset y
/// to put the next box at. Since every box in BFC has full width & layout
/// from top to bottom, we don't need to update width or offsett x.
pub(crate) fn update_context(root: &LayoutBox, context: &mut LayoutContext) {
    let rect = root.dimensions.margin_box();
    context.height += rect.height;
    context.offset_y += rect.height - root.dimensions.margin.bottom;

    if context.width < rect.width {
        context.width = rect.width;
    }
}