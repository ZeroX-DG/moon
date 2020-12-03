/// This module contains the definition of
/// the layout box, which is the component
/// that made up the layout tree.
use super::box_layout::ContainingBlock;
use super::box_model::Dimensions;
use super::is_block_container_box;
use style::render_tree::RenderNodeRef;
use tree::arena_tree::{Tree, TreeNode};
use std::ops::Deref;

#[derive(Debug)]
pub struct LayoutTree(Tree<LayoutBox>);

/// LayoutBox for the layout tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Type of this box (inline | block | anonymous)
    pub box_type: BoxType,

    /// Box model dimensions for this box
    pub dimensions: Dimensions,

    /// The render node that generate this box
    pub render_node: RenderNodeRef,

    /// The formatting context that this block establish
    pub formatting_context: Option<FormattingContext>,
}

/// Formatting context of each box
#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContext {
    Block,
    Inline,
}

/// Different box types for each layout box
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    /// Block-level box
    Block,

    /// Inline-level box
    Inline,

    /// Anonymous inline / Anonymous block box
    /// depending on the formatting context of
    /// the parent box
    Anonymous,
}

impl LayoutBox {
    pub fn new(node: RenderNodeRef, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: node,
            dimensions: Dimensions::default(),
            formatting_context: None,
        }
    }

    pub fn is_block_container_box(&self) -> bool {
        is_block_container_box(self)
    }

    pub fn box_model(&mut self) -> &mut Dimensions {
        &mut self.dimensions
    }

    pub fn as_containing_block(&self) -> ContainingBlock {
        let box_model = self.dimensions.clone();
        ContainingBlock {
            x: box_model.content.x - box_model.padding.left,
            y: box_model.content.y - box_model.padding.top,
            width: box_model.content.width,
            height: box_model.content.height,
            offset_x: box_model.content.x,
            offset_y: box_model.content.y,
            previous_margin_bottom: 0.0,
            collapsed_margins_vertical: 0.0,
        }
    }

    pub fn set_formatting_context(&mut self, context: FormattingContext) {
        self.formatting_context = Some(context);
    }
}

pub type LayoutBoxNode = TreeNode<LayoutBox>;

impl LayoutTree {
    pub fn new() -> Self {
        Self(Tree::new())
    }

    pub fn add_child(&mut self, parent: &LayoutBoxNode, child: LayoutBoxNode) {
        match parent.formatting_context {
            Some(FormattingContext::Block) => {
                // ensure that all child is block-level boxes
                match child.box_type {
                    BoxType::Block => {
                        if parent.box_type == BoxType::Inline {
                            // adding block box to inline box
                            // the inline box should be break in 2
                            // TODO: Implement this (we need to access parent box somehow)
                        }
                        parent.children.push(child.id);
                    }
                    BoxType::Anonymous => {
                        parent.children.push(child.id);
                    }
                    BoxType::Inline => {
                        let anonymous_block = self
                            .get_anonymous_block_for_inline(parent);
                        self.add_child(anonymous_block, child);
                    }
                }
            }
            Some(FormattingContext::Inline) => {
                // if a box has a inline formatting context
                // all of its children is inline boxes or anonymous inline boxes
                parent.children.push(child.id);
            }
            _ => {
                println!("This is just a box, it shouldn't contains anything");
            }
        }
    }

    fn get_anonymous_block_for_inline(&self, parent: &LayoutBoxNode) -> &LayoutBoxNode {
        let mut use_last_child = false;
        if let Some(last_child) = parent.children.last() {
            if let Some(last_child) = self.0.get_node(last_child) {
                if let BoxType::Anonymous = last_child.box_type {
                    if let Some(FormattingContext::Inline) = last_child.formatting_context {
                        use_last_child = true;
                    }
                }
            }
        }

        if use_last_child {
            return self.0.get_node(parent.children.last_mut().unwrap()).unwrap();
        }

        let mut new_box = LayoutBox::new(parent.render_node.clone(), BoxType::Anonymous);
        new_box.set_formatting_context(FormattingContext::Inline);

        let box_node = self.0.new_node(new_box);
        parent.children.push(box_node);
        self.0.get_node(&box_node).unwrap()
    }
}

impl Deref for LayoutTree {
    type Target = Tree<LayoutBox>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
