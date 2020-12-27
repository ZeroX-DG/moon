use crate::layout::LayoutContext;
use crate::layout_box::LayoutBox;

/// Update layout context for each box in inline formatting context.
pub(crate) fn update_context(root: &LayoutBox, context: &mut LayoutContext) {
    let rect = root.dimensions.margin_box();
    context.width += rect.width;
    context.offset_x += rect.width - root.dimensions.margin.right;

    if context.height < rect.height {
        context.height = rect.height;
    }
}