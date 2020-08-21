use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::ops::Deref;
use super::node_list::NodeList;

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
    node_type: Rc<RefCell<NodeType>>,
    parent_node: Option<WeakNodeRef<'a>>,
    first_child: Option<NodeRef<'a>>,
    last_child: Option<WeakNodeRef<'a>>,
    next_sibling: Option<NodeRef<'a>>,
    prev_sibling: Option<WeakNodeRef<'a>>,
    inner: Rc<RefCell<dyn NodeData + 'a>>
}

pub trait NodeData {}

pub struct NodeRef<'a>(Rc<RefCell<Node<'a>>>);
pub struct WeakNodeRef<'a>(Weak<RefCell<Node<'a>>>);

impl<'a> Node<'a> {
    pub fn new<D: NodeData + 'a>(node_type: NodeType, inner: D) -> Self {
        Self {
            node_type: Rc::new(RefCell::new(node_type)),
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

    pub fn parent(&self) -> Option<NodeRef<'a>> {
        let ref_self = self.borrow();
        match &ref_self.parent_node {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn next_sibling(&self) -> Option<NodeRef<'a>> {
        let ref_self = self.borrow();
        ref_self.next_sibling.clone()
    }

    pub fn prev_sibling(&self) -> Option<NodeRef<'a>> {
        let ref_self = self.borrow();
        match &ref_self.prev_sibling {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn first_child(&self) -> Option<NodeRef<'a>> {
        let ref_self = self.borrow();
        ref_self.first_child.clone()
    }

    pub fn last_child(&self) -> Option<NodeRef<'a>> {
        let ref_self = self.borrow();
        match &ref_self.last_child {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn node_type(&self) -> Rc<RefCell<NodeType>> {
        let ref_self = self.borrow();
        ref_self.node_type.clone()
    }

    pub fn child_nodes(&self) -> NodeList {
        NodeList::new(self.first_child())
    }

    pub fn inner(&self) -> Rc<RefCell<dyn NodeData + 'a>> {
        let ref_self = self.borrow();
        ref_self.inner.clone()
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
