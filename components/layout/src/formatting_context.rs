use super::box_model::{BoxComponent, Edge, Rect};
use super::layout_box::LayoutBox;
use style::value_processing::{Property, Value};
use style::values::display::{Display, InnerDisplayType};

use super::flow::block::BlockFormattingContext;
use super::flow::inline::InlineFormattingContext;

#[derive(Debug)]
pub struct BaseFormattingContext {
    pub offset_x: f32,
    pub offset_y: f32,
    pub width: f32,
    pub height: f32,
}

pub trait FormattingContext {
    fn layout(&mut self, boxes: Vec<&mut LayoutBox>, containing_block: &Rect) {
        for layout_box in boxes {
            self.calculate_width(layout_box);
            calculate_position(self.base(), layout_box, containing_block);
            layout_children(layout_box);
            apply_explicit_sizes(layout_box, containing_block);
            self.update_new_data(layout_box);
        }
    }

    fn calculate_width(&mut self, layout_box: &mut LayoutBox);

    fn base(&self) -> &BaseFormattingContext;

    fn update_new_data(&mut self, layout_box: &LayoutBox);
}

fn layout_children(layout_box: &mut LayoutBox) {
    let mut context = get_formatting_context(layout_box);
    let containing_block = &layout_box.dimensions.content;

    context.layout(layout_box.children.iter_mut().collect(), containing_block);

    if layout_box.is_height_auto() {
        layout_box.dimensions.set_height(context.base().height);
    }
}

fn get_formatting_context(layout_box: &LayoutBox) -> Box<dyn FormattingContext> {
    if let Some(node) = &layout_box.render_node {
        let node = node.borrow();
        let display = node.get_style(&Property::Display);
        let inner_display = match display.inner() {
            Value::Display(Display::Full(_, inner)) => inner,
            _ => unreachable!(),
        };

        match inner_display {
            InnerDisplayType::Flow => {
                if layout_box.children_are_inline() {
                    Box::new(InlineFormattingContext::new(&layout_box.dimensions.content))
                } else {
                    Box::new(BlockFormattingContext::new(&layout_box.dimensions.content))
                }
            }
            InnerDisplayType::FlowRoot => {
                Box::new(BlockFormattingContext::new(&layout_box.dimensions.content))
            }
            _ => unimplemented!("Unsupported display type: {:#?}", display),
        }
    } else {
        if layout_box.children_are_inline() {
            return Box::new(InlineFormattingContext::new(&layout_box.dimensions.content));
        }
        return Box::new(BlockFormattingContext::new(&layout_box.dimensions.content));
    }
}

fn calculate_position(
    base: &BaseFormattingContext,
    layout_box: &mut LayoutBox,
    containing_block: &Rect,
) {
    let render_node = layout_box.render_node.clone();
    let box_model = layout_box.box_model();

    if let Some(render_node) = render_node {
        let render_node = render_node.borrow();

        let margin_top = render_node
            .get_style(&Property::MarginTop)
            .to_px(containing_block.width);
        let margin_bottom = render_node
            .get_style(&Property::MarginBottom)
            .to_px(containing_block.width);

        let border_top = render_node
            .get_style(&Property::BorderTopWidth)
            .to_px(containing_block.width);
        let border_bottom = render_node
            .get_style(&Property::BorderBottomWidth)
            .to_px(containing_block.width);

        let padding_top = render_node
            .get_style(&Property::PaddingTop)
            .to_px(containing_block.width);
        let padding_bottom = render_node
            .get_style(&Property::PaddingBottom)
            .to_px(containing_block.width);

        box_model.set(BoxComponent::Margin, Edge::Top, margin_top);
        box_model.set(BoxComponent::Margin, Edge::Bottom, margin_bottom);

        box_model.set(BoxComponent::Padding, Edge::Top, padding_top);
        box_model.set(BoxComponent::Padding, Edge::Bottom, padding_bottom);

        box_model.set(BoxComponent::Border, Edge::Top, border_top);
        box_model.set(BoxComponent::Border, Edge::Bottom, border_bottom);
    }

    let content_area_x =
        base.offset_x + box_model.margin.left + box_model.border.left + box_model.padding.left;

    let content_area_y =
        base.offset_y + box_model.margin.top + box_model.border.top + box_model.padding.top;

    layout_box
        .box_model()
        .set_position(content_area_x, content_area_y);
}

fn apply_explicit_sizes(layout_box: &mut LayoutBox, containing_block: &Rect) {
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
