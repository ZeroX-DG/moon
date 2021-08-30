use crate::{
    box_model::Rect,
    flow::inline::InlineFormattingContext,
    layout_box::{children_are_inline, LayoutNodeId, LayoutTree},
};

use super::flow::block::BlockFormattingContext;
use style::value_processing::{Property, Value};
use style::values::display::{Display, InnerDisplayType};

#[derive(Debug, Clone)]
pub struct LayoutContext {
    pub viewport: Rect,
}

pub trait FormattingContext {
    fn run(&mut self, layout_node: &LayoutNodeId);

    fn layout_tree(&self) -> &LayoutTree;
    fn layout_tree_mut(&mut self) -> &mut LayoutTree;

    fn layout_content(&mut self, layout_node: &LayoutNodeId, layout_context: &LayoutContext) {
        let mut formatting_context =
            get_formatting_context(self.layout_tree_mut(), layout_node, layout_context);
        formatting_context.run(layout_node);
    }
}

fn get_formatting_context<'a>(
    tree: &'a mut LayoutTree,
    layout_node_id: &LayoutNodeId,
    layout_context: &'a LayoutContext,
) -> Box<dyn FormattingContext + 'a> {
    let layout_node = tree.get_node(layout_node_id);
    if layout_node.is_anonymous() {
        if children_are_inline(&tree, layout_node_id) {
            return Box::new(InlineFormattingContext::new(layout_context, tree));
        }
        return Box::new(BlockFormattingContext::new(layout_context, tree));
    }

    let node = layout_node.render_node().clone().unwrap();
    let node = node.borrow();

    let display = node.get_style(&Property::Display);
    let inner_display = match display.inner() {
        Value::Display(Display::Full(_, inner)) => inner,
        _ => unreachable!(),
    };

    match inner_display {
        InnerDisplayType::Flow => {
            if !children_are_inline(tree, &layout_node.id()) {
                Box::new(BlockFormattingContext::new(layout_context, tree))
            } else {
                Box::new(InlineFormattingContext::new(layout_context, tree))
            }
        }
        InnerDisplayType::FlowRoot => Box::new(BlockFormattingContext::new(layout_context, tree)),
        _ => unimplemented!("Unsupported display type: {:#?}", display),
    }
}
