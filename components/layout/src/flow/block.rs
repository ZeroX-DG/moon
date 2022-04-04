use crate::{box_model::BoxComponent, formatting_context::{LayoutContext, BaseFormattingContext, FormattingContext}, layout_box::LayoutBox};
use shared::primitive::edge::Edge;
use std::{rc::Rc, cell::RefCell};
use style::{property::Property, values::prelude::Position};

#[derive(Debug)]
pub struct BlockFormattingContext {
    last_sibling: RefCell<Option<Rc<LayoutBox>>>,
    base: BaseFormattingContext
}

impl FormattingContext for BlockFormattingContext {
    fn run(&self, context: &LayoutContext, layout_node: Rc<LayoutBox>) {
        if layout_node.is_block() && layout_node.parent().is_none() {
            self.layout_initial_block_box(context, layout_node);
            return;
        }

        self.compute_width(layout_node.clone());
        self.layout_block_level_children(context, layout_node.clone());
        self.compute_height(layout_node.clone());
    }

    fn base(&self) -> &BaseFormattingContext {
        &self.base
    }
}

impl BlockFormattingContext {
    pub fn new(base: BaseFormattingContext) -> Self {
        Self {
            last_sibling: RefCell::new(None),
            base
        }
    }

    fn layout_initial_block_box(&self, context: &LayoutContext, layout_node: Rc<LayoutBox>) {
        // Initial containing block has the dimensions of the viewport
        let width = context.viewport.width;
        let height = context.viewport.height;

        layout_node.set_content_width(width);
        layout_node.set_content_height(height);
        layout_node.set_offset(0., 0.);

        self.layout_block_level_children(context, layout_node);
    }

    fn layout_block_level_children(&self, context: &LayoutContext, layout_node: Rc<LayoutBox>) {
        for child in layout_node.children().iter() {
            if child.is_positioned(Position::Absolute) {
                continue;
            }
            self.compute_width(child.clone());
            self.place_box_in_flow(child.clone());

            child
                .formatting_context()
                .run(context, child.clone());

            if !child.children_are_inline() {
                self.compute_height(child.clone());
            }

            child.apply_explicit_sizes();

            if child.border_box_absolute().height > 0. {
                self.last_sibling.replace(Some(child.clone()));
            }
        }
    }

    fn place_box_in_flow(&self, layout_node: Rc<LayoutBox>) {
        self.apply_vertical_box_model_values(layout_node.clone());

        let box_model = layout_node.box_model().borrow();
        let x = box_model.margin_box().left + box_model.offset.left;

        let mut y = box_model.margin_box().top + box_model.offset.top;

        let mut previous_collapsed_margin_bottom = 0.;

        if let Some(sibling) = &*self.last_sibling.borrow() {
            previous_collapsed_margin_bottom = f32::max(
                previous_collapsed_margin_bottom,
                sibling.box_model().borrow().margin_box().bottom,
            );
            y += sibling.offset().y
                + sibling.content_size().height
                + sibling.box_model().borrow().border_box().bottom;

            if box_model.margin_box().top < 0. || previous_collapsed_margin_bottom < 0. {
                if box_model.margin_box().top < 0. && previous_collapsed_margin_bottom < 0. {
                    // When all margins are negative, the size of the collapsed margin is the smallest (most negative) margin.
                    let smallest_negative_margin =
                        f32::min(previous_collapsed_margin_bottom, box_model.margin_box().top);
                    y += smallest_negative_margin;
                } else {
                    // When negative margins are involved, the size of the collapsed margins
                    // is the sum of the largest positive margin and the smallest (most negative) negative margin.
                    let largest_positive_margin =
                        f32::max(previous_collapsed_margin_bottom, box_model.margin_box().top);
                    let smallest_negative_margin =
                        f32::min(previous_collapsed_margin_bottom, box_model.margin_box().top);

                    let margin_offset = largest_positive_margin + smallest_negative_margin;
                    y += margin_offset - box_model.margin_box().top;
                }
            } else if previous_collapsed_margin_bottom > box_model.margin_box().top {
                let final_collapsed_margin =
                    previous_collapsed_margin_bottom - box_model.margin_box().top;
                y += final_collapsed_margin;
            }
        }

        layout_node.set_offset(x, y);
    }

    fn compute_width(&self, layout_node: Rc<LayoutBox>) {
        let containing_block = layout_node.containing_block().content_size();

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
        if layout_node.is_block() && layout_node.is_non_replaced() {
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

        // 3.9 'Inline-block', non-replaced elements in normal flow
        if layout_node.is_inline_block() && layout_node.is_non_replaced() {
            // A computed value of 'auto' for 'margin-left' or 'margin-right' becomes a used value of '0'.
            if computed_margin_left.is_auto() {
                used_margin_left = 0.0;
            }
            if computed_margin_right.is_auto() {
                used_margin_right = 0.0;
            }

            // TODO: calculate fit-to-shrink width
        }

        // apply all calculated used values
        let mut box_model = layout_node.base.box_model.borrow_mut();

        layout_node.set_content_width(used_width);
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

    fn apply_vertical_box_model_values(&self, layout_node: Rc<LayoutBox>) {
        if layout_node.is_anonymous() {
            return;
        }

        let render_node = layout_node.render_node().unwrap();
        let containing_block = layout_node.containing_block().content_size();
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

        let mut box_model = layout_node.base.box_model.borrow_mut();
        box_model.set(BoxComponent::Margin, Edge::Top, margin_top);
        box_model.set(BoxComponent::Margin, Edge::Bottom, margin_bottom);
        box_model.set(BoxComponent::Padding, Edge::Top, padding_top);
        box_model.set(BoxComponent::Padding, Edge::Bottom, padding_bottom);
        box_model.set(BoxComponent::Border, Edge::Top, border_top);
        box_model.set(BoxComponent::Border, Edge::Bottom, border_bottom);
    }

    fn compute_height(&self, layout_node: Rc<LayoutBox>) {
        let height = self.compute_box_height(layout_node.clone());
        layout_node.set_content_height(height);
    }

    fn compute_box_height(&self, layout_node: Rc<LayoutBox>) -> f32 {
        if layout_node.is_anonymous() {
            return self.compute_auto_height(layout_node);
        }

        let containing_block = layout_node.containing_block().content_size();
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
            .fold(0.0, |acc, child| acc + child.margin_box_height())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting_context::{establish_context, FormattingContextType, LayoutContext};
    use crate::layout_box::BoxData;
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

        let layout_context = LayoutContext {
            viewport: Rect {
                x: 0.,
                y: 0.,
                width: 500.,
                height: 300.,
            },
        };

        let initial_block_box = Rc::new(LayoutBox::new_anonymous(BoxData::block_box()));
        establish_context(
            FormattingContextType::BlockFormattingContext,
            initial_block_box.clone(),
        );
        LayoutBox::add_child(initial_block_box.clone(), root.clone());

        initial_block_box
            .formatting_context()
            .run(&layout_context, initial_block_box.clone());

        assert_eq!(root.content_size().height, 40.);
        assert_eq!(root.content_size().width, layout_context.viewport.width);
    }
}
