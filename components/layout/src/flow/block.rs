use std::rc::Rc;
use style::{property::Property, values::prelude::Position};
use crate::{box_model::BoxComponent, formatting_context::LayoutContext, layout_box::LayoutBox};
use shared::primitive::edge::Edge;

pub struct BlockFormattingContext {
    previous_layout_y: f32,
    layout_context: Rc<LayoutContext>
}

impl BlockFormattingContext {
    pub fn new(layout_context: Rc<LayoutContext>) -> Self {
        Self {
            previous_layout_y: 0.,
            layout_context
        }
    }

    pub fn run(&mut self, layout_node: Rc<LayoutBox>) {
        if layout_node.is_block() && layout_node.parent().is_none() {
            self.layout_initial_block_box(layout_node);
            return;
        }

        self.layout_block_level_children(layout_node);
    }

    fn layout_initial_block_box(&mut self, layout_node: Rc<LayoutBox>) {
        // Initial containing block has the dimensions of the viewport
        let width = self.layout_context.viewport.width;
        let height = self.layout_context.viewport.height;

        layout_node.dimensions_mut().set_width(width);
        layout_node.dimensions_mut().set_height(height);
        layout_node.dimensions_mut().set_position(0., 0.);

        self.layout_block_level_children(layout_node);
    }

    fn layout_block_level_children(&mut self, layout_node: Rc<LayoutBox>) {
        for child in layout_node.children().iter() {
            if child.is_positioned(Position::Absolute) {
                continue;
            }

            self.compute_width(child.clone());

            if child.is_non_replaced() {
                self.compute_position_non_replaced(child.clone());
            }

            child.layout(self.layout_context.clone());

            if !child.children_are_inline() {
                self.compute_height(child.clone());
            }
            child.apply_explicit_sizes();

            let child_dimensions = child.dimensions();
            self.previous_layout_y += child_dimensions.margin_box().height;
        }
    }

    fn compute_width(&mut self, layout_node: Rc<LayoutBox>) {
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
        let mut box_model = layout_node.dimensions_mut();

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

    fn compute_height(&mut self, layout_node: Rc<LayoutBox>) {
        if layout_node.is_anonymous() {
            return;
        }

        let containing_block = layout_node
            .containing_block()
            .dimensions()
            .content_box();

        let height = self.compute_box_height(layout_node.clone());

        let render_node = layout_node.render_node().unwrap();

        let margin_top = render_node
            .get_style(&Property::MarginTop)
            .to_px(containing_block.width);
        let margin_bottom = render_node
            .get_style(&Property::MarginBottom)
            .to_px(containing_block.width);

        let padding_top = render_node
            .get_style(&Property::PaddingTop)
            .to_px(containing_block.width);
        let padding_bottom = render_node
            .get_style(&Property::PaddingBottom)
            .to_px(containing_block.width);

        let border_top = render_node
            .get_style(&Property::BorderTopWidth)
            .to_px(containing_block.width);
        let border_bottom = render_node
            .get_style(&Property::BorderBottomWidth)
            .to_px(containing_block.width);

        let mut box_model = layout_node.dimensions_mut();
        box_model.set_height(height);
        box_model.set(BoxComponent::Margin, Edge::Top, margin_top);
        box_model.set(BoxComponent::Margin, Edge::Bottom, margin_bottom);
        box_model.set(BoxComponent::Padding, Edge::Top, padding_top);
        box_model.set(BoxComponent::Padding, Edge::Bottom, padding_bottom);
        box_model.set(BoxComponent::Border, Edge::Top, border_top);
        box_model.set(BoxComponent::Border, Edge::Bottom, border_bottom);
    }

    fn compute_box_height(&self, layout_node: Rc<LayoutBox>) -> f32 {
        let containing_block = layout_node
            .containing_block()
            .dimensions()
            .content_box();
        let computed_height = layout_node
            .render_node()
            .unwrap()
            .get_style(&Property::Height);

        if computed_height.is_auto() {
            self.compute_auto_height(layout_node)
        } else {
            computed_height.to_px(containing_block.height)
        }
    }

    fn compute_auto_height(&self, layout_node: Rc<LayoutBox>) -> f32 {
        layout_node
            .children()
            .iter()
            .fold(0.0, |acc, child| {
                acc + child.dimensions().margin_box().height
            })
    }

    fn compute_position_non_replaced(&mut self, layout_node: Rc<LayoutBox>) {
        let containing_block = layout_node
            .containing_block()
            .dimensions()
            .content_box();

        let previous_layout_y = self.previous_layout_y;
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

        let content_area_x = containing_block.x
            + box_model.margin.left
            + box_model.border.left
            + box_model.padding.left;

        let content_area_y = containing_block.y
            + previous_layout_y
            + box_model.margin.top
            + box_model.border.top
            + box_model.padding.top;

        box_model.set_position(content_area_x, content_area_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting_context::LayoutContext;
    use crate::utils::*;
    use shared::primitive::*;
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

        let css = format!(
            "
        {}
        .box {{
            height: 10px;
        }}
        ",
            SHARED_CSS
        );

        let root = build_tree(dom, &css);

        let layout_context = Rc::new(LayoutContext {
            viewport: Rect {
                x: 0.,
                y: 0.,
                width: 500.,
                height: 300.,
            },
        });

        let mut formatting_context = BlockFormattingContext::new(layout_context.clone());

        formatting_context.run(root.clone());

        assert_eq!(root.dimensions().content_box().height, 40.);
        assert_eq!(root.dimensions().content_box().width, layout_context.viewport.width);
    }
}
