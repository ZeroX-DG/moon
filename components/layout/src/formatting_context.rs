use crate::box_model::Rect;

use super::flow::block::BlockFormattingContext;
use super::layout_box::LayoutNode;
use style::value_processing::{Property, Value};
use style::values::display::{Display, InnerDisplayType};

#[derive(Debug, Clone)]
pub struct LayoutContext {
    pub viewport: Rect
}

pub trait FormattingContext {
    fn run(&mut self, layout_node: &mut LayoutNode);

    fn layout_content(&self, layout_node: &mut LayoutNode, layout_context: LayoutContext) {
        let mut formatting_context = get_formatting_context(layout_node, layout_context);
        formatting_context.run(layout_node);
    }
}

fn get_formatting_context(
    layout_node: &LayoutNode,
    layout_context: LayoutContext,
) -> Box<dyn FormattingContext> {
    if layout_node.is_anonymous() {
        return Box::new(BlockFormattingContext::new(layout_context));
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
            if !layout_node.children_are_inline() {
                Box::new(BlockFormattingContext::new(layout_context))
            } else {
                panic!("no")
            }
        }
        InnerDisplayType::FlowRoot => Box::new(BlockFormattingContext::new(layout_context)),
        _ => unimplemented!("Unsupported display type: {:#?}", display),
    }
}

