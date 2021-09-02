use crate::property::Property;

use super::inheritable::INHERITABLES;
use super::value_processing::ValueRef;
use dom::dom_ref::NodeRef;
use std::collections::{HashMap, HashSet};
use tree::rctree::{TreeNodeRef, TreeNodeWeakRef};

pub type RenderNodeRef = TreeNodeRef<RenderNode>;
pub type RenderNodeWeak = TreeNodeWeakRef<RenderNode>;

#[derive(Debug)]
pub struct RenderTree {
    /// The root node of the render tree
    pub root: Option<RenderNodeRef>,
    /// The style cache to share style value and reduce style size
    pub style_cache: HashSet<ValueRef>,
}

/// A style node in the style tree
#[derive(Debug)]
pub struct RenderNode {
    /// A reference to the DOM node that uses this style
    pub node: NodeRef,
    /// A property HashMap containing computed styles
    pub properties: HashMap<Property, ValueRef>,
    /// Child style nodes
    pub children: Vec<RenderNodeRef>,
    /// Parent reference for inheritance
    pub parent_render_node: Option<RenderNodeWeak>,
}

impl RenderNode {
    /// Get style value of a property
    /// Ensure that the value return is a shared computed value
    pub fn get_style(&self, property: &Property) -> ValueRef {
        if let Some(value) = self.properties.get(property) {
            return value.clone();
        }

        if INHERITABLES.contains(property) {
            if let Some(parent) = &self.parent_render_node {
                if let Some(p) = parent.upgrade() {
                    return p.borrow().get_style(&property);
                }
            }
        }

        panic!("Oops, we should not reach here");
    }
}
