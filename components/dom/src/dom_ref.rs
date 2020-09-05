use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::ops::Deref;
use super::node::Node;

pub trait DOMObject {
    fn as_node(&self) -> &Node;
}

pub struct NodeRef(Rc<RefCell<dyn DOMObject>>);
pub struct WeakNodeRef(Weak<RefCell<dyn DOMObject>>);

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

impl WeakNodeRef {
    pub fn upgrade(self) -> Option<NodeRef> {
        match self.0.upgrade() {
            Some(node) => Some(NodeRef(node)),
            _ => None
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

