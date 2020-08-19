use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::ops::{Deref};
use super::node_list::NodeList;

use super::nodes::document::Document;
use super::nodes::element::Element;

pub enum NodeData {
    Document(Document),
    Element(Element)
}

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

pub struct Node {
    node_type: Rc<RefCell<NodeType>>,
    parent_node: Option<WeakNodeRef>,
    first_child: Option<NodeRef>,
    last_child: Option<WeakNodeRef>,
    next_sibling: Option<NodeRef>,
    prev_sibling: Option<WeakNodeRef>,
    data: Rc<RefCell<NodeData>>
}

pub struct NodeRef(Rc<RefCell<Node>>);
pub struct WeakNodeRef(Weak<RefCell<Node>>);

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

    pub fn downgrade(self) -> WeakNodeRef {
        WeakNodeRef(Rc::downgrade(&self.0))
    }

    pub fn parent(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.parent_node {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn next_sibling(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        ref_self.next_sibling.clone()
    }

    pub fn prev_sibling(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.prev_sibling {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn first_child(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        ref_self.first_child.clone()
    }

    pub fn last_child(&self) -> Option<NodeRef> {
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

    pub fn data(&self) -> Rc<RefCell<NodeData>> {
        let ref_self = self.borrow();
        ref_self.data.clone()
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
