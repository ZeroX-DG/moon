use super::box_layout::ContainingBlock;
/// This module contains the definition of
/// the layout box, which is the component
/// that made up the layout tree.
use super::box_model::Dimensions;
use super::is_block_container_box;
use style::render_tree::RenderNodeRef;

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

    /// The parent formatting context that this element participate in
    pub parent_formatting_context: Option<FormattingContext>,

    /// The children of this box
    pub children: Vec<LayoutBox>,
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
            parent_formatting_context: None,
            children: Vec::new(),
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
            x: box_model.content.x,
            y: box_model.content.y,
            width: box_model.content.width,
            height: box_model.content.height,
            offset_x: 0.0,
            offset_y: 0.0,
            previous_margin_bottom: 0.0,
            collapsed_margins_vertical: 0.0,
        }
    }

    pub fn add_child(&mut self, child: LayoutBox) {
        match self.formatting_context {
            Some(FormattingContext::Block) => {
                // ensure that all child is block-level boxes
                match child.box_type {
                    BoxType::Block => {
                        if self.box_type == BoxType::Inline {
                            // adding block box to inline box
                            // the inline box should be break in 2
                            // TODO: Implement this (we need to access parent box somehow)
                        }
                        self.children.push(child);
                    }
                    BoxType::Anonymous => {
                        self.children.push(child);
                    }
                    BoxType::Inline => {
                        self.get_anonymous_block_for_inline().add_child(child);
                    }
                }
            }
            Some(FormattingContext::Inline) => {
                // if a box has a inline formatting context
                // all of its children is inline boxes or anonymous inline boxes
                self.children.push(child);
            }
            _ => {
                println!("This is just a box, it shouldn't contains anything");
            }
        }
    }

    fn get_anonymous_block_for_inline(&mut self) -> &mut LayoutBox {
        let mut use_last_child = false;
        if let Some(last_child) = self.children.last() {
            if let BoxType::Anonymous = last_child.box_type {
                if let Some(FormattingContext::Inline) = last_child.formatting_context {
                    use_last_child = true;
                }
            }
        }

        if use_last_child {
            return self.children.last_mut().unwrap();
        }
        let mut new_box = LayoutBox::new(self.render_node.clone(), BoxType::Anonymous);
        new_box.set_formatting_context(FormattingContext::Inline);
        self.children.push(new_box);
        self.children.last_mut().unwrap()
    }

    pub fn set_formatting_context(&mut self, context: FormattingContext) {
        self.formatting_context = Some(context);
    }

    pub fn set_parent_formatting_context(&mut self, context: FormattingContext) {
        self.parent_formatting_context = Some(context);
    }
}
