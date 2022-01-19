use crate::{property::Property, value_processing::StyleCache};

use super::inheritable::INHERITABLES;
use super::value_processing::ValueRef;
use dom::node::Node;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::{Rc, Weak}};

#[derive(Debug)]
pub struct RenderTree {
    /// The root node of the render tree
    pub root: Option<Rc<RenderNode>>,
    /// The style cache to share style value and reduce style size
    pub style_cache: StyleCache,
}

/// A style node in the style tree
pub struct RenderNode {
    /// A reference to the DOM node that uses this style
    pub node: Rc<Node>,
    /// A property HashMap containing computed styles
    pub properties: HashMap<Property, ValueRef>,
    /// Child style nodes
    pub children: RefCell<Vec<Rc<RenderNode>>>,
    /// Parent reference for inheritance
    pub parent_render_node: Option<Weak<RenderNode>>,
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
                    return p.get_style(&property);
                }
            }
        }

        panic!("Oops, we should not reach here");
    }
}

impl Debug for RenderNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.node)
    }
}

impl RenderTree {
    pub fn to_str(&self) -> String {
        let mut result = String::new();

        fn print_node(result: &mut String, node: Rc<RenderNode>) {
            result.push_str(&format!("{:?}\n", node.node));

            for child in node.children.borrow().iter() {
                print_node(result, child.clone());
            }
        }

        if let Some(root) = &self.root {
            print_node(&mut result, root.clone());
        }

        result
    }
}
