use std::any::Any;

use style::{property::Property, render_tree::RenderNodeRef};

use crate::{
    box_model::{BoxComponent, Dimensions, Edge},
    formatting_context::{FormattingContext, LayoutContext},
    layout_box::{
        apply_explicit_sizes, get_containing_block, LayoutBox, LayoutNode, LayoutNodeId, LayoutTree,
    },
};

#[derive(Debug)]
pub struct InlineBox {
    node: Option<RenderNodeRef>,
    dimensions: Dimensions,
}

impl LayoutBox for InlineBox {
    fn is_inline(&self) -> bool {
        true
    }

    fn is_block(&self) -> bool {
        false
    }

    fn render_node(&self) -> Option<RenderNodeRef> {
        self.node.clone()
    }

    fn friendly_name(&self) -> &str {
        "InlineBox"
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

impl InlineBox {
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

#[derive(Debug)]
pub struct LineBox {
    fragments: Vec<LayoutNodeId>,
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

    pub fn add_fragment(&mut self, child: &LayoutNode) {
        let child_size = child.dimensions().margin_box();
        self.fragments.push(child.id());
        self.width += child_size.width;
        self.height = if self.height > child_size.height {
            self.height
        } else {
            child_size.height
        };
    }

    pub fn fragments(&self) -> &[LayoutNodeId] {
        &self.fragments
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

pub struct InlineFormattingContext<'a> {
    layout_context: &'a LayoutContext,
    layout_tree: &'a mut LayoutTree,
}

impl<'a> InlineFormattingContext<'a> {
    pub fn new(layout_context: &'a LayoutContext, layout_tree: &'a mut LayoutTree) -> Self {
        Self {
            layout_context,
            layout_tree,
        }
    }

    fn calculate_width(&mut self, layout_node_id: &LayoutNodeId) {
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
        let box_model = layout_node.dimensions_mut();
        box_model.set_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
    }

    fn apply_vertical_spacing(&mut self, layout_node_id: &LayoutNodeId) {
        let containing_block = self
            .layout_tree()
            .get_node(&get_containing_block(self.layout_tree(), layout_node_id))
            .dimensions()
            .content_box();

        let layout_node = self.layout_tree_mut().get_node_mut(layout_node_id);
        let render_node = layout_node.render_node();
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
    }
}

impl<'a> FormattingContext for InlineFormattingContext<'a> {
    fn run(&mut self, layout_node_id: &LayoutNodeId) {
        let mut line_boxes = Vec::new();

        line_boxes.push(LineBox::new());

        let containing_block = self
            .layout_tree
            .get_node(&layout_node_id)
            .dimensions()
            .content_box();

        let parent_width = containing_block.width;

        let children = self
            .layout_tree()
            .children(layout_node_id)
            .iter()
            .copied()
            .collect::<Vec<usize>>();

        for child_id in children {
            self.calculate_width(&child_id);
            self.layout_content(&child_id, &self.layout_context);
            self.apply_vertical_spacing(&child_id);
            apply_explicit_sizes(self.layout_tree_mut(), &child_id);

            let child = self.layout_tree.get_node(&child_id);

            let child_width = child.dimensions().content.width;

            let line_box = line_boxes.last_mut().unwrap();

            let new_line_box_width = line_box.width() + child_width;

            if new_line_box_width > parent_width {
                line_boxes.push(LineBox::new());
            }

            let line_box = line_boxes.last_mut().unwrap();
            line_box.add_fragment(child);
        }

        let mut offset_y = 0.;

        for line in &line_boxes {
            let mut offset_x = 0.;

            for fragment in line.fragments() {
                let fragment = self.layout_tree.get_node_mut(fragment);

                let x = containing_block.x + offset_x + fragment.dimensions().margin.left;

                let y = containing_block.y + offset_y + fragment.dimensions().margin.top;

                fragment.dimensions_mut().set_position(x, y);
                offset_x += fragment.dimensions().margin_box().width;
            }

            offset_y += line.height();
        }

        self.layout_tree
            .get_node_mut(layout_node_id)
            .dimensions_mut()
            .set_height(offset_y);
    }

    fn layout_tree(&self) -> &LayoutTree {
        &self.layout_tree
    }

    fn layout_tree_mut(&mut self) -> &mut LayoutTree {
        &mut self.layout_tree
    }
}
