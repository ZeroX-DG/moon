use crate::box_model::{BoxComponent, Edge, Rect};
use crate::layout_box::LayoutBox;
use style::value_processing::Property;

pub fn compute_position(root: &mut LayoutBox, containing_block: &Rect) {
    let render_node = root.render_node.clone();
    let box_model = root.box_model();

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
        0.0 + box_model.margin.left + box_model.border.left + box_model.padding.left;

    let content_area_y = 0.0 + box_model.border.top + box_model.padding.top;

    root.box_model()
        .set_position(content_area_x, content_area_y);
}
