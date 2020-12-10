/// This module is responsible for the box generation of elements
/// in the render tree. In other words, this module transforms
/// render tree to layout tree and prepare for layouting process.
use style::{render_tree::RenderNodeRef, values::display::{InnerDisplayType, OuterDisplayType}};
use style::value_processing::{Property, Value};
use style::values::display::Display;
use crate::layout_box::{BaseBox, BlockLevelBox, FormattingContext, FormattingContextRef, InlineLevelBox, LayoutBox, inline::InlineBox};
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
    if node.borrow().node.is::<dom::text::Text>() {
        // TODO: Support text nodes when we support text run boxes
        return None;
    }

    let mut layout_box = match build_box(&node, &formatting_context) {
        Some(lb) => lb,
        None => return None
    };

    for child in &node.borrow().children {
        handle_build_child_box(&mut layout_box, child);
    }

    Some(layout_box)
}

fn handle_build_child_box(parent: &mut LayoutBox, child: &RenderNodeRef) {
    let display_value = child.borrow().get_style(&Property::Display);
    
    match display_value.inner() {
        Value::Display(Display::Full(full)) => match (full.outer(), full.inner()) {
            (OuterDisplayType::Inline, InnerDisplayType::Flow) => {
            }
            _ => {
                if let Some(child_box) = build_layout_tree_recursive(
                    child.clone(),
                    Some(parent.formatting_context())
                ) {
                    parent.add_child(child_box)
                }
            }
        }
        _ => {}
    }
}

/// Build the layout box base on the element
/// display value and parent formatting context.
fn build_box(
    node: &RenderNodeRef,
    parent_context: &Option<FormattingContextRef>
) -> Option<LayoutBox> {
    let display_value = node.borrow().get_style(&Property::Display);

    match display_value.inner() {
        Value::Display(Display::Full(full)) => match (full.outer(), full.inner()) {
            // if an element is display 'block', it generates block formatting ctx
            (OuterDisplayType::Block, InnerDisplayType::Flow) => {
                let formatting_context = match parent_context {
                    Some(ctx) => match ***ctx {
                        FormattingContext::Block => ctx.clone(),
                        _ => FormattingContextRef::new_block_context()
                    },
                    None => FormattingContextRef::new_block_context()
                };
                let block_box = BlockContainerBox {
                    base: BaseBox::new(node.clone(), formatting_context)
                };
                let block_container = BlockLevelBox::BlockContainerBox(block_box);
                let layout_box = LayoutBox::BlockLevelBox(block_container);
                Some(layout_box)
            }
            (OuterDisplayType::Inline, InnerDisplayType::Flow) => {
                // The contents of an inline box participate in the same
                // inline formatting context as the inline box itself.
                let formatting_context = match parent_context {
                    Some(ctx) => match ***ctx {
                        FormattingContext::Inline => ctx.clone(),
                        _ => panic!("Parent context is not inline")
                    },
                    _ => panic!("Parent context for inline box is missing")
                };
                let inline_box = InlineBox {
                    base: BaseBox::new(node.clone(), formatting_context)
                };
                let inline_level_box = InlineLevelBox::InlineBox(inline_box);
                let layout_box = LayoutBox::InlineLevelBox(inline_level_box);
                Some(layout_box)
            }
            _ => None
        }
        _ => None
    }
}
