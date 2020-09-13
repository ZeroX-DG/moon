use super::element::Element;
use super::node::Node;
use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

pub trait DOMObject {
    fn as_node(&self) -> &Node;
    fn as_node_mut(&mut self) -> &mut Node;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_element(&self) -> Option<&Element>;
    fn as_element_mut(&mut self) -> Option<&mut Element>;
}

#[derive(Debug)]
pub struct NodeRef(Rc<RefCell<dyn DOMObject>>);
#[derive(Debug)]
pub struct WeakNodeRef(Weak<RefCell<dyn DOMObject>>);

impl core::fmt::Display for dyn DOMObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DOMObject at {:?}", self as *const dyn DOMObject)
    }
}

impl core::fmt::Debug for dyn DOMObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DOMObject at {:?}", self as *const dyn DOMObject)
    }
}

impl Deref for NodeRef {
    type Target = RefCell<dyn DOMObject>;

    fn deref(&self) -> &RefCell<dyn DOMObject> {
        &*self.0
    }
}

impl Clone for WeakNodeRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Clone for NodeRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl PartialEq for NodeRef {
    fn eq(&self, other: &NodeRef) -> bool {
        self.as_ptr().eq(&other.as_ptr())
    }
}

impl WeakNodeRef {
    pub fn upgrade(self) -> Option<NodeRef> {
        match self.0.upgrade() {
            Some(node) => Some(NodeRef(node)),
            _ => None,
        }
    }
}

impl NodeRef {
    pub fn new<D: DOMObject + 'static>(node: D) -> Self {
        Self(Rc::new(RefCell::new(node)))
    }

    pub fn downgrade(self) -> WeakNodeRef {
        WeakNodeRef(Rc::downgrade(&self.0))
    }
}
