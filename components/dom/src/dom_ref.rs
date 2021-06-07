use super::node::Node;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

pub struct NodeRef(Rc<RefCell<Node>>);
pub struct WeakNodeRef(Weak<RefCell<Node>>);

impl core::fmt::Debug for NodeRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NodeRef({:#?})", self.borrow().deref())
    }
}

impl core::fmt::Debug for WeakNodeRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "WeakNodeRef({:#?})",
            self.0.upgrade().unwrap().borrow().deref()
        )
    }
}

impl Deref for NodeRef {
    type Target = RefCell<Node>;

    fn deref(&self) -> &RefCell<Node> {
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

    pub fn empty() -> Self {
        Self(Weak::new())
    }
}

impl NodeRef {
    pub fn new(node: Node) -> Self {
        Self(Rc::new(RefCell::new(node)))
    }

    pub fn downgrade(self) -> WeakNodeRef {
        WeakNodeRef(Rc::downgrade(&self.0))
    }

    pub fn is_element(&self) -> bool {
        self.0.borrow().as_element_opt().is_some()
    }

    pub fn is_document(&self) -> bool {
        self.0.borrow().as_document_opt().is_some()
    }

    pub fn is_text(&self) -> bool {
        self.0.borrow().as_text_opt().is_some()
    }

    pub fn is_comment(&self) -> bool {
        self.0.borrow().as_comment_opt().is_some()
    }
}

