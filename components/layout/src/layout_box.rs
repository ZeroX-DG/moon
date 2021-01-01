/// This module contains the definition of
/// the layout box, which is the component
/// that made up the layout tree.
use super::box_model::Dimensions;
use super::formatting_context::FormattingContext;
use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::{Display, InnerDisplayType};
use style::values::float::Float;
use style::values::position::Position;

use super::flow::block::BlockFormattingContext;
use super::flow::inline::InlineFormattingContext;

/// LayoutBox for the layout tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Type of this box (inline | block | anonymous)
    pub box_type: BoxType,

    /// Box model dimensions for this box
    pub dimensions: Dimensions,

    /// The render node that generate this box
    pub render_node: Option<RenderNodeRef>,

    /// Indicate if this box only contain inline
    pub children_are_inline: bool,

    /// The children of this box
    pub children: Vec<LayoutBox>,
}

/// Different box types for each layout box
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    /// Block-level box
    Block,

    /// Inline-level box
    Inline,
}

impl LayoutBox {
    pub fn new(node: RenderNodeRef, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: Some(node),
            dimensions: Dimensions::default(),
            children_are_inline: false,
            children: Vec::new(),
        }
    }

    pub fn new_anonymous(box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: None,
            dimensions: Dimensions::default(),
            children_are_inline: false,
            children: Vec::new(),
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.render_node.is_none()
    }

    pub fn is_inline(&self) -> bool {
        self.box_type == BoxType::Inline
    }

    pub fn is_block(&self) -> bool {
        self.box_type == BoxType::Block
    }

    pub fn is_float(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().get_style(&Property::Float).inner() {
                Value::Float(Float::None) => false,
                _ => true,
            },
            _ => false,
        }
    }

    pub fn is_non_replaced(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().node.borrow().as_element() {
                Some(e) => match e.tag_name().as_str() {
                    "video" | "image" | "img" | "canvas" => false,
                    _ => true,
                },
                _ => true,
            },
            _ => true,
        }
    }

    // TODO: change to the correct behavior of inline block
    pub fn is_inline_block(&self) -> bool {
        self.is_inline()
    }

    // TODO: change to the correct behavior to detect normal flow
    pub fn is_in_normal_flow(&self) -> bool {
        true
    }

    pub fn is_absolutely_positioned(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().get_style(&Property::Position).inner() {
                Value::Position(Position::Absolute) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn box_model(&mut self) -> &mut Dimensions {
        &mut self.dimensions
    }

    pub fn to_string(&self) -> String {
        dump_layout_tree(&self, 0, &LayoutDumpSpecificity::Structure)
    }

    #[allow(dead_code)]
    pub(crate) fn dump(&self, specificity: &LayoutDumpSpecificity) -> String {
        dump_layout_tree(&self, 0, specificity)
    }

    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    pub fn set_children_inline(&mut self, value: bool) {
        self.children_are_inline = value;
    }

    pub fn children_are_inline(&self) -> bool {
        self.children_are_inline
    }

    pub fn is_height_auto(&self) -> bool {
        if let Some(node) = &self.render_node {
            let computed_height = node.borrow().get_style(&Property::Height);

            return computed_height.is_auto();
        }
        return true;
    }

    pub fn layout(&mut self) {
        let mut context = self.establish_formatting_context();
        context.layout(self.children.iter_mut().collect(), &self.dimensions.content);

        if self.is_height_auto() {
            self.dimensions.set_height(context.base().height);
        }
    }

    fn establish_formatting_context(&self) -> Box<dyn FormattingContext> {
        if let Some(node) = &self.render_node {
            let node = node.borrow();
            let display = node.get_style(&Property::Display);
            let inner_display = match display.inner() {
                Value::Display(Display::Full(_, inner)) => inner,
                _ => unreachable!(),
            };

            match inner_display {
                InnerDisplayType::Flow => {
                    if self.children_are_inline() {
                        Box::new(InlineFormattingContext::new(&self.dimensions.content))
                    } else {
                        Box::new(BlockFormattingContext::new(&self.dimensions.content))
                    }
                }
                _ => unimplemented!("Unsupported display type: {:#?}", display),
            }
        } else {
            if self.children_are_inline() {
                return Box::new(InlineFormattingContext::new(&self.dimensions.content));
            }
            return Box::new(BlockFormattingContext::new(&self.dimensions.content));
        }
    }
}

#[allow(dead_code)]
pub(crate) enum LayoutDumpSpecificity {
    Structure,
    StructureAndDimensions,
}

fn dump_layout_tree(root: &LayoutBox, level: usize, specificity: &LayoutDumpSpecificity) -> String {
    let mut result = String::new();
    let child_nodes = &root.children;

    let dimensions_str = match specificity {
        LayoutDumpSpecificity::Structure => String::new(),
        LayoutDumpSpecificity::StructureAndDimensions => format!(
            "(x: {}| y: {}| w: {}| h: {})",
            root.dimensions.content.x,
            root.dimensions.content.y,
            root.dimensions.content.width,
            root.dimensions.content.height
        ),
    };

    if let Some(node) = &root.render_node {
        result.push_str(&format!(
            "{}[{:?}] {:#?} {}\n",
            "  ".repeat(level),
            root.box_type,
            node.borrow().node,
            dimensions_str
        ));
    } else {
        result.push_str(&format!(
            "{}[Anonymous {:?}] {}\n",
            "  ".repeat(level),
            root.box_type,
            dimensions_str
        ));
    }

    for node in child_nodes {
        result.push_str(&dump_layout_tree(node, level + 1, specificity));
    }
    return result;
}
