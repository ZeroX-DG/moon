use std::rc::Rc;

use style::property::Property;
use crate::{box_model::BoxComponent, formatting_context::LayoutContext, layout_box::LayoutBox};
use shared::primitive::edge::Edge;

#[derive(Debug)]
pub struct InlineBox {
    line_boxes: Vec<LineBox>
}

impl InlineBox {
    pub fn new() -> Self {
        Self {
            line_boxes: Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct LineBox {
    fragments: Vec<Rc<LayoutBox>>,
    width: f32,
    height: f32,
}

impl LineBox {
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
            width: 0.,
            height: 0.,
        }
    }

    pub fn add_fragment(&mut self, child: Rc<LayoutBox>) {
        let child_size = child.dimensions().margin_box();
        self.fragments.push(child);
        self.width += child_size.width;
        self.height = if self.height > child_size.height {
            self.height
        } else {
            child_size.height
        };
    }

    pub fn fragments(&self) -> &[Rc<LayoutBox>] {
        &self.fragments
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

pub struct InlineFormattingContext {
    layout_context: Rc<LayoutContext>,
}

impl InlineFormattingContext {
    pub fn new(layout_context: Rc<LayoutContext>) -> Self {
        Self {
            layout_context,
        }
    }

    pub fn run(&mut self, layout_node: Rc<LayoutBox>) {
        let mut line_boxes = Vec::new();

        line_boxes.push(LineBox::new());

        let containing_block = layout_node
            .dimensions()
            .content_box();

        let parent_width = containing_block.width;

        for child in layout_node.children().iter() {
            self.calculate_width(child.clone());
            child.formatting_context().run(self.layout_context.clone(), child.clone());
            self.apply_vertical_spacing(child.clone());
            child.apply_explicit_sizes();

            let child_width = child.dimensions().margin_box().width;

            let line_box = line_boxes.last_mut().unwrap();

            let new_line_box_width = line_box.width() + child_width;

            if new_line_box_width > parent_width {
                line_boxes.push(LineBox::new());
            }

            let line_box = line_boxes.last_mut().unwrap();
            line_box.add_fragment(child.clone());
        }

        let mut offset_y = 0.;

        for line in &line_boxes {
            let mut offset_x = 0.;

            for fragment in line.fragments() {
                let x = containing_block.x + offset_x + fragment.dimensions().margin.left;

                let y = containing_block.y + offset_y + fragment.dimensions().margin.top;

                fragment.dimensions_mut().set_position(x, y);
                offset_x += fragment.dimensions().margin_box().width;
            }

            offset_y += line.height();
        }

        layout_node
            .dimensions_mut()
            .set_height(offset_y);
    }

    fn calculate_width(&mut self, layout_node: Rc<LayoutBox>) {
        let containing_block = layout_node
            .containing_block()
            .dimensions()
            .content_box();

        let render_node = match layout_node.render_node() {
            Some(node) => node.clone(),
            _ => return,
        };

        let computed_width = render_node.get_style(&Property::Width);
        let computed_margin_left = render_node.get_style(&Property::MarginLeft);
        let computed_margin_right = render_node.get_style(&Property::MarginRight);
        let containing_width = containing_block.width;

        let mut used_width = computed_width.to_px(containing_width);
        let mut used_margin_left = computed_margin_left.to_px(containing_width);
        let mut used_margin_right = computed_margin_right.to_px(containing_width);

        if layout_node.is_non_replaced() && !layout_node.is_inline_block() {
            used_width = 0.0;
            used_margin_left = 0.0;
            used_margin_right = 0.0;
        }

        if layout_node.is_non_replaced() && layout_node.is_inline_block() {
            if computed_margin_left.is_auto() {
                used_margin_left = 0.0;
            }
            if computed_margin_right.is_auto() {
                used_margin_right = 0.0;
            }
            if computed_width.is_auto() {
                // TODO: Support auto width when we have shrink-to-fit width
            }
        }

        // apply all calculated used values
        let mut box_model = layout_node.dimensions_mut();
        box_model.set_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
    }

    fn apply_vertical_spacing(&mut self, layout_node: Rc<LayoutBox>) {
        let containing_block = layout_node
            .containing_block()
            .dimensions()
            .content_box();

        let render_node = layout_node.render_node();
        let mut box_model = layout_node.dimensions_mut();

        if let Some(render_node) = render_node {
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
    }
}

