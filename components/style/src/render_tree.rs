use crate::{property::Property, value_processing::StyleCache};

use super::inheritable::INHERITABLES;
use super::value_processing::ValueRef;
use dom::node::NodePtr;
use shared::tree_node::{TreeNode, TreeNodeHooks};
use std::{
    collections::HashMap,
    fmt::Debug,
    ops::Deref,
};

pub struct RenderNodePtr(pub TreeNode<RenderNode>);

#[derive(Debug)]
pub struct RenderTree {
    /// The root node of the render tree
    pub root: Option<RenderNodePtr>,
    /// The style cache to share style value and reduce style size
    pub style_cache: StyleCache,
}

/// A style node in the style tree
pub struct RenderNode {
    /// A reference to the DOM node that uses this style
    pub node: NodePtr,
    /// A property HashMap containing computed styles
    pub properties: HashMap<Property, ValueRef>,
}

impl TreeNodeHooks<RenderNode> for RenderNode {}
impl Debug for RenderNodePtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl Deref for RenderNodePtr {
    type Target = TreeNode<RenderNode>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Clone for RenderNodePtr {
    fn clone(&self) -> Self {
        RenderNodePtr(self.0.clone())
    }
}

impl RenderNodePtr {
    /// Get style value of a property
    /// Ensure that the value return is a shared computed value
    pub fn get_style(&self, property: &Property) -> ValueRef {
        if let Some(value) = self.properties.get(property) {
            return value.clone();
        }

        if INHERITABLES.contains(property) {
            if let Some(parent) = self.parent() {
                return RenderNodePtr(parent).get_style(&property);
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

        fn print_node(result: &mut String, node: RenderNodePtr) {
            result.push_str(&format!("{:?}\n", node.node));

            node.for_each_child(|child| {
                print_node(result, RenderNodePtr(child));
            })
        }

        if let Some(root) = &self.root {
            print_node(&mut result, root.clone());
        }

        result
    }
}
