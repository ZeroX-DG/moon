use super::dom_ref::{WeakNodeRef, NodeRef};

pub struct Node {
    pub parent_node: Option<WeakNodeRef>,
    pub first_child: Option<NodeRef>,
    pub last_child: Option<WeakNodeRef>,
    pub next_sibling: Option<NodeRef>,
    pub prev_sibling: Option<WeakNodeRef>,
    pub owner_document: Option<WeakNodeRef>,
}

