use super::box_model::{BoxComponent, Edge, Rect};
use super::layout_box::LayoutBox;
use style::value_processing::Property;

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
            self.calculate_position(layout_box, containing_block);
            layout_box.layout();
            self.apply_explicit_sizes(layout_box, containing_block);
            self.update_new_data(layout_box);
        }
    }

    fn calculate_width(&mut self, layout_box: &mut LayoutBox);

    fn base(&self) -> &BaseFormattingContext;

    fn update_new_data(&mut self, layout_box: &LayoutBox);

    fn calculate_position(&mut self, layout_box: &mut LayoutBox, containing_block: &Rect) {
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

        let content_area_x = self.base().offset_x
            + box_model.margin.left
            + box_model.border.left
            + box_model.padding.left;

        let content_area_y = self.base().offset_y
            + box_model.margin.top
            + box_model.border.top
            + box_model.padding.top;

        layout_box
            .box_model()
            .set_position(content_area_x, content_area_y);
    }

    fn apply_explicit_sizes(&mut self, layout_box: &mut LayoutBox, containing_block: &Rect) {
        if layout_box.is_inline() {
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
}
