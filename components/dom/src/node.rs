use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::ops::Deref;

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

pub struct Node<'a> {
    pub(crate) node_type: NodeType,
    pub(crate) parent_node: Option<WeakNodeRef<'a>>,
    pub(crate) first_child: Option<NodeRef<'a>>,
    pub(crate) last_child: Option<WeakNodeRef<'a>>,
    pub(crate) next_sibling: Option<NodeRef<'a>>,
    pub(crate) prev_sibling: Option<WeakNodeRef<'a>>,
    pub(crate) inner: Rc<RefCell<dyn NodeData + 'a>>
}

pub trait NodeData {}

pub struct NodeRef<'a>(pub(crate) Rc<RefCell<Node<'a>>>);
pub struct WeakNodeRef<'a>(pub(crate) Weak<RefCell<Node<'a>>>);

impl<'a> Node<'a> {
    pub fn new<D: NodeData + 'a>(node_type: NodeType, inner: D) -> Self {
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

impl<'a> Deref for NodeRef<'a> {
    type Target = RefCell<Node<'a>>;

    fn deref(&self) -> &RefCell<Node<'a>> {
        &*self.0
    }
}

impl<'a> Clone for WeakNodeRef<'a> {
    fn clone(&self) -> Self {
        Self(self.0.clone())        
    }
}

impl<'a> Clone for NodeRef<'a> {
    fn clone(&self) -> Self {
        Self(self.0.clone())        
    }
}

impl<'a> NodeRef<'a> {
    pub fn new(node: Node<'a>) -> Self {
        Self(Rc::new(RefCell::new(node)))
    }

    pub fn new_node<D: NodeData + 'a>(node_type: NodeType, inner: D) -> Self {
        NodeRef::new(Node::new(node_type, inner))
    }

    pub fn downgrade(self) -> WeakNodeRef<'a> {
        WeakNodeRef(Rc::downgrade(&self.0))
    }

    pub fn inner(&self) -> &Rc<RefCell<Node<'a>>> {
        &self.0
    }
}

impl<'a> WeakNodeRef<'a> {
    pub fn upgrade(self) -> Option<NodeRef<'a>> {
        match self.0.upgrade() {
            Some(node) => Some(NodeRef(node)),
            _ => None
        }
    }
}

#[cfg(test)]
mod test {
}
