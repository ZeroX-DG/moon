use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::ops::Deref;

use super::nodes::{Document, Comment, DocumentType, Element};

#[derive(Debug, Clone)]
pub enum NodeType {
    Element = 1,
    Text = 3,
    CDataSection = 4,
    ProcessingInstruction = 7,
    Comment = 8,
    Document = 9,
    DocumentType = 10,
    DocumentFragment = 11
}

pub enum NodeInner {
    Document(Document),
    Comment(Comment),
    DocumentType(DocumentType),
    Element(Element)
}

pub struct Node {
    pub(crate) node_type: NodeType,
    pub(crate) parent_node: Option<WeakNodeRef>,
    pub(crate) first_child: Option<NodeRef>,
    pub(crate) last_child: Option<WeakNodeRef>,
    pub(crate) next_sibling: Option<NodeRef>,
    pub(crate) prev_sibling: Option<WeakNodeRef>,
    pub(crate) inner: Rc<RefCell<NodeInner>>
}

pub struct NodeRef(pub(crate) Rc<RefCell<Node>>);
pub struct WeakNodeRef(pub(crate) Weak<RefCell<Node>>);

impl Node {
    pub fn new(node_type: NodeType, inner: NodeInner) -> Self {
        Self {
            node_type,
            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            inner: Rc::new(RefCell::new(inner))
        }
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

impl NodeRef {
    pub fn new(node: Node) -> Self {
        Self(Rc::new(RefCell::new(node)))
    }

    pub fn new_node(node_type: NodeType, inner: NodeInner) -> Self {
        NodeRef::new(Node::new(node_type, inner))
    }

    pub fn downgrade(self) -> WeakNodeRef {
        WeakNodeRef(Rc::downgrade(&self.0))
    }

    pub fn inner(&self) -> Rc<RefCell<NodeInner>> {
        let ref_self = self.borrow();
        ref_self.inner.clone()
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

#[cfg(test)]
mod test {
}
