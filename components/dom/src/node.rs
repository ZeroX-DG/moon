use super::dom_ref::{WeakNodeRef, NodeRef};

#[derive(Debug)]
pub struct Node {
    parent_node: Option<WeakNodeRef>,
    first_child: Option<NodeRef>,
    last_child: Option<WeakNodeRef>,
    next_sibling: Option<NodeRef>,
    prev_sibling: Option<WeakNodeRef>,
    owner_document: Option<WeakNodeRef>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            owner_document: None
        }
    }

    pub fn set_document(&mut self, doc: WeakNodeRef) {
        self.owner_document = Some(doc);
    }

    pub fn first_child(&self) -> Option<NodeRef> {
        self.first_child.clone()
    }

    pub fn last_child(&self) -> Option<NodeRef> {
        match &self.last_child {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn next_sibling(&self) -> Option<NodeRef> {
        self.next_sibling.clone()
    }

    pub fn prev_sibling(&self) -> Option<NodeRef> {
        match &self.prev_sibling {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn owner_document(&self) -> Option<NodeRef> {
        match &self.owner_document {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn append_child(parent: NodeRef, child: NodeRef) {
        if let Some(last_child) = parent.borrow().as_node().last_child() {
            last_child.borrow_mut().as_node_mut().next_sibling = Some(child.clone());
        }

        child.borrow_mut().as_node_mut().parent_node = Some(parent.clone().downgrade());

        parent.borrow_mut().as_node_mut().last_child = Some(child.downgrade());
    }

    pub fn insert_before(parent: NodeRef, child: NodeRef, ref_child: Option<NodeRef>) {
        if let Some(ref_child) = ref_child {
            let mut child_ref = child.borrow_mut();
            if let Some(prev_sibling) = ref_child.borrow().as_node().prev_sibling() {
                prev_sibling.borrow_mut().as_node_mut().next_sibling = Some(child.clone());
                child_ref.as_node_mut().prev_sibling = Some(prev_sibling.clone().downgrade());
            }
            child_ref.as_node_mut().next_sibling = Some(ref_child.clone());
            ref_child.borrow_mut().as_node_mut().prev_sibling = Some(child.clone().downgrade());
        } else {
            Node::append_child(parent, child);
        }
    }
}
