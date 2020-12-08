/// This module is responsible for the box generation of elements
/// in the render tree. In other words, this module transforms
/// render tree to layout tree and prepare for layouting process.
use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::Display;
use crate::layout_box::{BaseBox, BlockLevelBox, FormattingContextRef, LayoutBox};
use crate::layout_box::block::BlockContainerBox;

/// Build the layout tree from the root render node tree
/// which is the root `html` tag/element.
pub fn build_layout_tree(root: RenderNodeRef) -> Option<LayoutBox> {
    build_layout_tree_recursive(root, None)
}

/// Recursively build the layout tree.
fn build_layout_tree_recursive(
    node: RenderNodeRef,
    formatting_context: Option<FormattingContextRef>
) -> Option<LayoutBox> {
    // Determine the box type that the node generate
    // Determine the formatting context that the box establish
    // Build the layout tree of the children of the nodes
    // Add those children boxes into the current box
    // Return the current box
    let layout_box = build_box(&node, &formatting_context);

    layout_box
}

/// Build the layout box base on the element
/// display value and parent formatting context.
fn build_box(
    node: &RenderNodeRef,
    parent_context: &Option<FormattingContextRef>
) -> Option<LayoutBox> {
    if node.borrow().node.is::<dom::text::Text>() {
        // TODO: Support text nodes when we support text run boxes
        return None;
    }

    let display_value = node.borrow().get_style(&Property::Display);

    match display_value.inner() {
        // if an element is display 'block', it generates block box
        Value::Display(Display::Block) => {
            let formatting_context = match parent_context {
                Some(ctx) => *ctx.clone(),
                None => FormattingContextRef::new_block_context()
            };
            let block_box = BlockContainerBox {
                base: BaseBox::new(node.clone(), formatting_context)
            };
            let block_container = BlockLevelBox::BlockContainerBox(block_box);
            let layout_box = LayoutBox::BlockLevelBox(block_container);
            Some(layout_box)
        }
        _ => None
    }
}
