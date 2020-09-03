use crate::node::{NodeType, NodeRef, WeakNodeRef};
use crate::node_list::NodeList;

pub trait Node {
    fn node_type(&self) -> NodeType;
    fn child_nodes(&self) -> NodeList;
    fn first_child(&self) -> Option<NodeRef>;
    fn last_child(&self) -> Option<NodeRef>;
    fn next_sibling(&self) -> Option<NodeRef>;
    fn prev_sibling(&self) -> Option<NodeRef>;
    fn owner_document(&self) -> Option<NodeRef>;
    fn set_document(&self, document: WeakNodeRef);
    fn append_child(&self, child: NodeRef);
    fn insert_before(&self, child: NodeRef, ref_child: Option<NodeRef>);
}

impl Node for NodeRef {
    fn node_type(&self) -> NodeType {
        let ref_self = self.borrow();
        ref_self.node_type.clone()
    }

    fn first_child(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        ref_self.first_child.clone()
    }

    fn last_child(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.last_child {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    fn next_sibling(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        ref_self.next_sibling.clone()
    }

    fn prev_sibling(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.prev_sibling {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    fn child_nodes(&self) -> NodeList {
        NodeList::new(self.first_child())
    }

    fn owner_document(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.owner_document {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    fn set_document(&self, document: WeakNodeRef) {
        let mut ref_self = self.borrow_mut();
        ref_self.owner_document = Some(document);
    }

    fn append_child(&self, child: NodeRef) {
        let mut ref_self = self.borrow_mut();

        if let Some(last_child) = self.last_child() {
            last_child.borrow_mut().next_sibling = Some(child.clone());
        }

        child.borrow_mut().parent_node = Some(self.clone().downgrade());

        ref_self.last_child = Some(child.downgrade());
    }

    fn insert_before(&self, child: NodeRef, ref_child: Option<NodeRef>) {
        if let Some(ref_child) = ref_child {
            let mut child_ref = child.borrow_mut();
            if let Some(prev_sibling) = ref_child.prev_sibling() {
                prev_sibling.borrow_mut().next_sibling = Some(child.clone());
                child_ref.prev_sibling = Some(prev_sibling.clone().downgrade());
            }
            child_ref.next_sibling = Some(ref_child.clone());
            ref_child.borrow_mut().prev_sibling = Some(child.clone().downgrade());
        } else {
            self.append_child(child);
        }
    }
}
