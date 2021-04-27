use super::box_model::Rect;
use super::layout_box::LayoutBox;
use style::value_processing::{Property, Value};
use style::values::display::{Display, InnerDisplayType};

use super::flow::block::BlockFormattingContext;
use super::flow::inline::InlineFormattingContext;

pub trait FormattingContext {
    fn layout(&mut self, boxes: Vec<&mut LayoutBox>) -> f32;

    fn get_containing_block(&mut self) -> &mut LayoutBox;
}

pub fn layout_children(layout_box: &mut LayoutBox) {
    let mut context = get_formatting_context(layout_box);

    let height = context.layout(layout_box.children.iter_mut().collect());

    if layout_box.is_height_auto() {
        layout_box.dimensions.set_height(height);
    }
}

fn get_formatting_context(layout_box: &mut LayoutBox) -> Box<dyn FormattingContext> {
    if layout_box.render_node.is_none() {
        if layout_box.children_are_inline() {
            return Box::new(InlineFormattingContext::new(layout_box));
        }
        return Box::new(BlockFormattingContext::new(layout_box));
    }

    let node = layout_box.render_node.clone().unwrap();
    let node = node.borrow();

    let display = node.get_style(&Property::Display);
    let inner_display = match display.inner() {
        Value::Display(Display::Full(_, inner)) => inner,
        _ => unreachable!(),
    };

    match inner_display {
        InnerDisplayType::Flow => {
            if layout_box.children_are_inline() {
                Box::new(InlineFormattingContext::new(layout_box))
            } else {
                Box::new(BlockFormattingContext::new(layout_box))
            }
        }
        InnerDisplayType::FlowRoot => Box::new(BlockFormattingContext::new(layout_box)),
        _ => unimplemented!("Unsupported display type: {:#?}", display),
    }
}

pub fn apply_explicit_sizes(layout_box: &mut LayoutBox, containing_block: &Rect) {
    if layout_box.is_inline() && !layout_box.is_inline_block() {
        return;
    }

    if let Some(render_node) = &layout_box.render_node {
        let computed_width = render_node.borrow().get_style(&Property::Width);
        let computed_height = render_node.borrow().get_style(&Property::Height);

        if !computed_width.is_auto() {
            let used_width = computed_width.to_px(containing_block.width);
            layout_box.box_model().set_width(used_width);
        }

        if !computed_height.is_auto() {
            let used_height = computed_height.to_px(containing_block.height);
            layout_box.box_model().set_height(used_height);
        }
    }
}
