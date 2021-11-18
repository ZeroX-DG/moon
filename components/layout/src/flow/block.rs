use std::any::Any;

use style::{property::Property, render_tree::RenderNodeRef, values::prelude::Position};

use crate::{
    box_model::{BoxComponent, Dimensions},
    formatting_context::{FormattingContext, LayoutContext},
    layout_box::{
        apply_explicit_sizes, children_are_inline, get_containing_block, LayoutBox, LayoutNodeId,
        LayoutTree,
    },
};

use shared::primitive::edge::Edge;

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
            dimensions: Default::default(),
        }
    }

    pub fn new_anonymous() -> Self {
        Self {
            node: None,
            dimensions: Default::default(),
        }
    }
}

pub struct BlockFormattingContext<'a> {
    layout_context: &'a LayoutContext,
    previous_layout_y: f32,
    layout_tree: &'a mut LayoutTree,
}

impl<'a> FormattingContext for BlockFormattingContext<'a> {
    fn run(&mut self, layout_node_id: &LayoutNodeId) {
        let layout_node = self.layout_tree_mut().get_node_mut(layout_node_id);
        if layout_node.is_block() && layout_node.parent().is_none() {
            self.layout_initial_block_box(layout_node_id);
            return;
        }

        self.layout_block_level_children(layout_node_id);
    }

    fn layout_tree(&self) -> &LayoutTree {
        &self.layout_tree
    }

    fn layout_tree_mut(&mut self) -> &mut LayoutTree {
        &mut self.layout_tree
    }
}

impl<'a> BlockFormattingContext<'a> {
    pub fn new(layout_context: &'a LayoutContext, layout_tree: &'a mut LayoutTree) -> Self {
        Self {
            layout_context,
            previous_layout_y: 0.,
            layout_tree,
        }
    }

    fn layout_initial_block_box(&mut self, layout_node_id: &LayoutNodeId) {
        // Initial containing block has the dimensions of the viewport
        let width = self.layout_context.viewport.width;
        let height = self.layout_context.viewport.height;

        let block_box = self.layout_tree_mut().get_node_mut(layout_node_id);
        let block_box_dimensions = block_box.dimensions_mut();
        block_box_dimensions.set_width(width);
        block_box_dimensions.set_height(height);
        block_box_dimensions.set_position(0., 0.);

        self.layout_block_level_children(layout_node_id);
    }

    fn layout_block_level_children(&mut self, layout_node_id: &LayoutNodeId) {
        let children = self
            .layout_tree()
            .children(layout_node_id)
            .iter()
            .copied()
            .collect::<Vec<usize>>();

        for child in children {
            if self
                .layout_tree()
                .get_node(&child)
                .is_positioned(Position::Absolute)
            {
                continue;
            }

            self.compute_width(&child);

            if self.layout_tree().get_node(&child).is_non_replaced() {
                self.compute_position_non_replaced(&child);
            }

            self.layout_content(&child, &self.layout_context);

            if !children_are_inline(&self.layout_tree(), &child) {
                self.compute_height(&child);
            }
            apply_explicit_sizes(self.layout_tree_mut(), &child);

            let child_dimensions = self.layout_tree_mut().get_node_mut(&child).dimensions();
            self.previous_layout_y += child_dimensions.margin_box().height;
        }
    }

    fn compute_width(&mut self, layout_node_id: &LayoutNodeId) {
        let containing_block = self
            .layout_tree()
            .get_node(&get_containing_block(self.layout_tree(), layout_node_id))
            .dimensions()
            .content_box();

        let layout_node = self.layout_tree_mut().get_node_mut(layout_node_id);
        let render_node = match layout_node.render_node() {
            Some(node) => node.clone(),
            _ => return,
        };

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

    fn compute_height(&mut self, layout_node_id: &LayoutNodeId) {
        if self.layout_tree().get_node(layout_node_id).is_anonymous() {
            return;
        }

        let containing_block = self
            .layout_tree()
            .get_node(&get_containing_block(self.layout_tree(), layout_node_id))
            .as_ref()
            .dimensions()
            .content_box();

        let height = self.compute_box_height(layout_node_id);

        let layout_node = self.layout_tree_mut().get_node_mut(layout_node_id);

        let render_node = layout_node.render_node().clone().unwrap();
        let render_node = render_node.borrow();

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

        let box_model = layout_node.dimensions_mut();
        box_model.set_height(height);
        box_model.set(BoxComponent::Margin, Edge::Top, margin_top);
        box_model.set(BoxComponent::Margin, Edge::Bottom, margin_bottom);
        box_model.set(BoxComponent::Padding, Edge::Top, padding_top);
        box_model.set(BoxComponent::Padding, Edge::Bottom, padding_bottom);
        box_model.set(BoxComponent::Border, Edge::Top, border_top);
        box_model.set(BoxComponent::Border, Edge::Bottom, border_bottom);
    }

    fn compute_box_height(&self, layout_node_id: &LayoutNodeId) -> f32 {
        let containing_block = self
            .layout_tree()
            .get_node(&get_containing_block(self.layout_tree(), layout_node_id))
            .dimensions()
            .content_box();
        let computed_height = self
            .layout_tree()
            .get_node(layout_node_id)
            .render_node()
            .clone()
            .unwrap()
            .borrow()
            .get_style(&Property::Height);

        if computed_height.is_auto() {
            self.compute_auto_height(layout_node_id)
        } else {
            computed_height.to_px(containing_block.height)
        }
    }

    fn compute_auto_height(&self, layout_node_id: &LayoutNodeId) -> f32 {
        self.layout_tree()
            .children(layout_node_id)
            .iter()
            .map(|child| self.layout_tree().get_node(child))
            .fold(0.0, |acc, child| {
                acc + child.dimensions().margin_box().height
            })
    }

    fn compute_position_non_replaced(&mut self, layout_node_id: &LayoutNodeId) {
        let containing_block = self
            .layout_tree()
            .get_node(&get_containing_block(self.layout_tree(), layout_node_id))
            .dimensions()
            .content_box();

        let previous_layout_y = self.previous_layout_y;
        let render_node = self
            .layout_tree()
            .get_node(layout_node_id)
            .render_node()
            .clone();
        let box_model = self
            .layout_tree_mut()
            .get_node_mut(layout_node_id)
            .dimensions_mut();

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

        let mut layout_tree = build_tree(dom, &css);
        let root = layout_tree.root().unwrap();

        let layout_context = LayoutContext {
            viewport: Rect {
                x: 0.,
                y: 0.,
                width: 500.,
                height: 300.,
            },
        };

        let initial_block_box = layout_tree.set_root(Box::new(BlockBox::new_anonymous()));
        layout_tree.add_child_by_id(&initial_block_box, &root);

        let mut formatting_context = BlockFormattingContext::new(&layout_context, &mut layout_tree);

        formatting_context.run(&initial_block_box);

        //println!("{}", layout_box.dump(&LayoutDumpSpecificity::StructureAndDimensions));

        assert_eq!(layout_tree.get_node(&root).dimensions().content.height, 40.);
        assert_eq!(
            layout_tree
                .get_node(&initial_block_box)
                .dimensions()
                .content
                .width,
            layout_context.viewport.width
        );
        //assert_eq!(layout_box.offset_y, 40.);
    }
}
