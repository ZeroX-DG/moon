use std::any::Any;

use style::{render_tree::RenderNodeRef, value_processing::Property, values::prelude::Position};

use crate::{box_model::{BoxComponent, Dimensions, Edge}, formatting_context::{FormattingContext, LayoutContext}, layout_box::{LayoutBox, LayoutNode}};

#[derive(Debug)]
pub struct BlockBox {
    node: Option<RenderNodeRef>,
    dimensions: Dimensions,
}

impl LayoutBox for BlockBox {
    fn is_inline(&self) -> bool {
        false
    }

    fn is_block(&self) -> bool {
        true
    }

    fn render_node(&self) -> Option<RenderNodeRef> {
        self.node.clone()
    }

    fn friendly_name(&self) -> &str {
        "BlockBox"
    }

    fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }

    fn dimensions_mut(&mut self) -> &mut Dimensions {
        &mut self.dimensions
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl BlockBox {
    pub fn new(node: RenderNodeRef) -> Self {
        Self {
            node: Some(node),
            dimensions: Default::default()
        }
    }

    pub fn new_anonymous() -> Self {
        Self {
            node: None,
            dimensions: Default::default()
        }
    }

    pub fn is_initial(&self) -> bool {
        match &self.node {
            Some(node) => node.borrow().parent_render_node.is_none(),
            _ => false
        }
    }
}

pub struct BlockFormattingContext {
    layout_context: LayoutContext,
    previous_layout_y: f32,
}

impl FormattingContext for BlockFormattingContext {
    fn run(&mut self, layout_node: &mut LayoutNode) {
        if layout_node.as_box::<BlockBox>().is_initial() {
            self.layout_initial_block_box(layout_node);
            return;
        }

        self.compute_position_non_replaced(layout_node);
        self.compute_width(layout_node);
        self.layout_block_level_children(layout_node);
        self.compute_height(layout_node);
    }
}

impl BlockFormattingContext {
    pub fn new(layout_context: LayoutContext) -> Self {
        Self {
            layout_context,
            previous_layout_y: 0.,
        }
    }

    fn layout_initial_block_box(&mut self, block_box: &mut LayoutNode) {
        // Initial containing block has the dimensions of the viewport
        let block_box_dimensions = block_box.dimensions_mut();
        block_box_dimensions.set_width(self.layout_context.viewport.width);
        block_box_dimensions.set_height(self.layout_context.viewport.height);

        self.layout_block_level_children(block_box);
    }

    fn layout_block_level_children(&mut self, block_box: &mut LayoutNode) {
        let mut content_height = 0.;
        let mut content_width = 0.;

        for child in block_box.children_mut() {
            if child.is_positioned(Position::Absolute) {
                continue;
            }

            self.compute_width(child);
            self.layout_content(child, self.layout_context.clone());
            self.compute_height(child);

            if child.is_non_replaced() {
                self.compute_position_non_replaced(child);
            }

            let child_borrow = child;
            let child_dimensions = child_borrow.dimensions();

            let child_bottom = child_dimensions.content_box().y
                + child_dimensions.content_box().height
                + child_dimensions.padding.bottom
                + child_dimensions.margin.bottom;

            let child_width = child_dimensions.margin_box().width;

            content_height = if content_height > child_bottom {
                content_height
            } else {
                child_bottom
            };
            content_width = if content_width > child_width {
                content_width
            } else {
                child_width
            };

            self.previous_layout_y = content_height;
        }

        if block_box.is_style_auto(&Property::Width) {
            block_box.dimensions_mut().set_width(content_width);
        }
    }

    fn compute_width(&self, layout_node: &mut LayoutNode) {
        let render_node = match layout_node.render_node() {
            Some(node) => node.clone(),
            _ => return,
        };

        let containing_block = layout_node
            .containing_block()
            .as_ref()
            .dimensions()
            .content_box();

        let render_node = render_node.borrow();
        let computed_width = render_node.get_style(&Property::Width);
        let computed_margin_left = render_node.get_style(&Property::MarginLeft);
        let computed_margin_right = render_node.get_style(&Property::MarginRight);
        let computed_border_left = render_node.get_style(&Property::BorderLeftWidth);
        let computed_border_right = render_node.get_style(&Property::BorderRightWidth);
        let computed_padding_left = render_node.get_style(&Property::PaddingLeft);
        let computed_padding_right = render_node.get_style(&Property::PaddingRight);
        let containing_width = containing_block.width;

        let box_width = computed_margin_left.to_px(containing_width)
            + computed_border_left.to_px(containing_width)
            + computed_padding_left.to_px(containing_width)
            + computed_width.to_px(containing_width)
            + computed_padding_right.to_px(containing_width)
            + computed_border_right.to_px(containing_width)
            + computed_margin_right.to_px(containing_width);

        let mut used_width = computed_width.to_px(containing_width);
        let mut used_margin_left = computed_margin_left.to_px(containing_width);
        let mut used_margin_right = computed_margin_right.to_px(containing_width);

        // 3. block-level, non-replaced elements in normal flow
        if layout_node.is_non_replaced() {
            if !computed_width.is_auto() && box_width > containing_width {
                if computed_margin_left.is_auto() {
                    used_margin_left = 0.0;
                }
                if computed_margin_right.is_auto() {
                    used_margin_right = 0.0;
                }
            }

            let underflow = containing_width - box_width;

            match (
                computed_width.is_auto(),
                computed_margin_left.is_auto(),
                computed_margin_right.is_auto(),
            ) {
                // If all of the above have a computed value other than 'auto',
                // the values are said to be "over-constrained" and one of the
                // used values will have to be different from its computed value.
                // If the 'direction' property of the containing block has the
                // value 'ltr', the specified value of 'margin-right' is ignored
                // and the value is calculated so as to make the equality true.
                // If the value of 'direction' is 'rtl', this happens to
                // 'margin-left' instead.
                (false, false, false) => {
                    // TODO: support direction rtl
                    used_margin_right = computed_margin_right.to_px(containing_width) + underflow;
                }
                // If there is exactly one value specified as 'auto',
                // its used value follows from the equality.
                (false, true, false) => {
                    used_margin_left = underflow;
                }
                (false, false, true) => {
                    used_margin_right = underflow;
                }
                // If 'width' is set to 'auto', any other 'auto' values become '0'
                // and 'width' follows from the resulting equality.
                (true, _, _) => {
                    if computed_margin_left.is_auto() {
                        used_margin_left = 0.0;
                    }
                    if computed_margin_right.is_auto() {
                        used_margin_right = 0.0;
                    }

                    if underflow >= 0. {
                        used_width = underflow;
                    } else {
                        used_width = 0.;
                        used_margin_right =
                            computed_margin_right.to_px(containing_width) + underflow;
                    }
                }
                // If both 'margin-left' and 'margin-right' are 'auto', their
                // used values are equal. This horizontally centers the element
                // with respect to the edges of the containing block.
                (false, true, true) => {
                    let half_underflow = underflow / 2.;
                    used_margin_left = half_underflow;
                    used_margin_right = half_underflow;
                }
            }
        }

        // apply all calculated used values
        let box_model = layout_node.dimensions_mut();
        
        box_model.set_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
        box_model.set(
            BoxComponent::Padding,
            Edge::Left,
            computed_padding_left.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Padding,
            Edge::Right,
            computed_padding_right.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Border,
            Edge::Left,
            computed_border_left.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Border,
            Edge::Right,
            computed_border_right.to_px(containing_width),
        );
    }

    fn compute_height(&self, layout_node: &mut LayoutNode) {
        if layout_node.is_anonymous() {
            return;
        }
        let height = self.compute_box_height(layout_node);

        let containing_block = layout_node
            .containing_block()
            .as_ref()
            .dimensions()
            .content_box();

        let render_node = layout_node.render_node().clone().unwrap();
        let render_node = render_node.borrow();

        let margin_top = render_node.get_style(&Property::MarginTop).to_px(containing_block.width);
        let margin_bottom = render_node.get_style(&Property::MarginBottom).to_px(containing_block.width);

        let padding_top = render_node.get_style(&Property::PaddingTop).to_px(containing_block.width);
        let padding_bottom = render_node.get_style(&Property::PaddingBottom).to_px(containing_block.width);

        let border_top = render_node.get_style(&Property::BorderTopWidth).to_px(containing_block.width);
        let border_bottom = render_node.get_style(&Property::BorderBottomWidth).to_px(containing_block.width);


        let box_model = layout_node.dimensions_mut();
        box_model.set_height(height);
        box_model.set(BoxComponent::Margin, Edge::Top, margin_top);
        box_model.set(BoxComponent::Margin, Edge::Bottom, margin_bottom);
        box_model.set(
            BoxComponent::Padding,
            Edge::Top,
            padding_top,
        );
        box_model.set(
            BoxComponent::Padding,
            Edge::Bottom,
            padding_bottom,
        );
        box_model.set(
            BoxComponent::Border,
            Edge::Top,
            border_top,
        );
        box_model.set(
            BoxComponent::Border,
            Edge::Bottom,
            border_bottom,
        );
    }

    fn compute_box_height(&self, layout_node: &mut LayoutNode) -> f32 {
        let containing_block = layout_node
            .containing_block()
            .as_ref()
            .dimensions()
            .content_box();
        let computed_height = layout_node
            .render_node()
            .clone()
            .unwrap()
            .borrow()
            .get_style(&Property::Height);
        
        if computed_height.is_auto() {
            self.compute_auto_height(layout_node)
        } else {
            computed_height.to_px(containing_block.height)
        }
    }

    fn compute_auto_height(&self, layout_node: &LayoutNode) -> f32 {
        if layout_node.children_are_inline() {
            return layout_node.children().iter().fold(0.0, |max_height, child| {
                let child_height = child.dimensions().margin_box().height;

                if max_height < child_height {
                    child_height
                } else {
                    max_height
                }
            });
        }
        layout_node.children().iter().fold(0.0, |acc, child| {
            acc + child.dimensions().margin_box().height
        })
    }

    fn compute_position_non_replaced(&self, layout_node: &mut LayoutNode) {
        let containing_block = layout_node
            .containing_block()
            .as_ref()
            .dimensions()
            .content_box();

        let render_node = layout_node.render_node().clone();
        let box_model = layout_node.dimensions_mut();

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

        let content_area_x = containing_block.x
            + box_model.margin.left
            + box_model.border.left
            + box_model.padding.left;

        let content_area_y = containing_block.y
            + self.previous_layout_y
            + box_model.margin.top
            + box_model.border.top
            + box_model.padding.top;

        box_model.set_position(content_area_x, content_area_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{box_model::Rect, utils::*};
    use test_utils::dom_creator::*;

    #[test]
    fn test_block_layout_simple() {
        let document = document();
        let dom = element(
            "div",
            document.clone(),
            vec![
                element("div.box", document.clone(), vec![]),
                element("div.box", document.clone(), vec![]),
                element("div.box", document.clone(), vec![]),
                element("div.box", document.clone(), vec![]),
            ],
        );

        let layout_box = build_tree(dom, SHARED_CSS);

        let layout_context = LayoutContext {
            viewport: Rect {
                x: 0.,
                y: 0.,
                width: 500.,
                height: 300.,
            }
        };

        let mut initial_block_box = LayoutNode::new(BlockBox::new_anonymous());
        initial_block_box.add_child(layout_box);

        let mut formatting_context = BlockFormattingContext::new(layout_context);

        formatting_context.run(&mut initial_block_box);

        //println!("{}", layout_box.dump(&LayoutDumpSpecificity::StructureAndDimensions));

        assert_eq!(initial_block_box.dimensions().content.height, 40.);
        //assert_eq!(layout_box.offset_y, 40.);
    }
}
