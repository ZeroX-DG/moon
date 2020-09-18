use super::dom_ref::{NodeRef, WeakNodeRef};

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
            owner_document: None,
        }
    }

    /// Set the owner document for node
    pub fn set_document(&mut self, doc: WeakNodeRef) {
        self.owner_document = Some(doc);
    }

    /// First child of the node
    pub fn first_child(&self) -> Option<NodeRef> {
        self.first_child.clone()
    }

    /// Last child of the node
    pub fn last_child(&self) -> Option<NodeRef> {
        match &self.last_child {
            Some(node) => node.clone().upgrade(),
            _ => None,
        }
    }

    /// Next sibling of the node
    pub fn next_sibling(&self) -> Option<NodeRef> {
        self.next_sibling.clone()
    }

    /// Previous sibling of the node
    pub fn prev_sibling(&self) -> Option<NodeRef> {
        match &self.prev_sibling {
            Some(node) => node.clone().upgrade(),
            _ => None,
        }
    }

    /// Parent of the node
    pub fn parent(&self) -> Option<NodeRef> {
        match &self.parent_node {
            Some(node) => node.clone().upgrade(),
            _ => None,
        }
    }

    /// Owner document of the node
    pub fn owner_document(&self) -> Option<NodeRef> {
        match &self.owner_document {
            Some(node) => node.clone().upgrade(),
            _ => None,
        }
    }

    pub fn detach(&mut self) {
        self.parent_node = None;
        self.prev_sibling = None;
        self.next_sibling = None;
    }

    /// Append a child node to a parent node
    ///
    /// **Ensure that:**
    /// 1. Last child of the parent is the child
    /// 2. First child of the parent is also the child if the parent has only 1 child
    /// 3. The child parent is this parent
    /// 4. The next-to-last child of the parent next sibling is the child if the parent has more
    ///    than 1 child
    pub fn append_child(parent: NodeRef, child: NodeRef) {
        if let Some(last_child) = parent.borrow().as_node().last_child() {
            last_child.borrow_mut().as_node_mut().next_sibling = Some(child.clone());
            child.borrow_mut().as_node_mut().prev_sibling = Some(last_child.clone().downgrade());
        }

        child.borrow_mut().as_node_mut().parent_node = Some(parent.clone().downgrade());

        if parent.borrow().as_node().first_child().is_none() {
            parent.borrow_mut().as_node_mut().first_child = Some(child.clone());
        }

        parent.borrow_mut().as_node_mut().last_child = Some(child.downgrade());
    }

    /// Insert a child node to a parent node before a reference child node
    ///
    /// **Ensure that:**
    /// 1. The previous sibling of reference child is the child node if reference child is present
    /// 2. If the reference child is not present, append the child to the parent
    /// 3. The first child of the parent is the child if the parent contains only the reference
    ///    child
    /// 4. The next sibling of the child before the reference child is the inserted child
    /// 5. The next sibling of the inserted child is the reference child
    /// 6. The previous sibling of the inserted node is the node before the reference node
    /// 7. The parent of the inserted child is the parent
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn append_child_first_child() {
        let parent = NodeRef::new(Node::new());
        let child = NodeRef::new(Node::new());

        Node::append_child(parent.clone(), child.clone());

        assert_eq!(parent.borrow().as_node().first_child(), Some(child.clone()));
        assert_eq!(parent.borrow().as_node().last_child(), Some(child.clone()));
        assert_eq!(child.borrow().as_node().parent(), Some(parent.clone()));
        assert_eq!(child.borrow().as_node().prev_sibling(), None);
        assert_eq!(child.borrow().as_node().next_sibling(), None);
    }

    #[test]
    fn append_child_normal() {
        let parent = NodeRef::new(Node::new());
        let child1 = NodeRef::new(Node::new());
        let child2 = NodeRef::new(Node::new());

        Node::append_child(parent.clone(), child1.clone());
        Node::append_child(parent.clone(), child2.clone());

        assert_eq!(
            parent.borrow().as_node().first_child(),
            Some(child1.clone())
        );
        assert_eq!(parent.borrow().as_node().last_child(), Some(child2.clone()));
        assert_eq!(
            child1.borrow().as_node().next_sibling(),
            Some(child2.clone())
        );
        assert_eq!(
            child2.borrow().as_node().prev_sibling(),
            Some(child1.clone())
        );
        assert_eq!(child1.borrow().as_node().prev_sibling(), None);
        assert_eq!(child2.borrow().as_node().next_sibling(), None);

        assert_eq!(child1.borrow().as_node().parent(), Some(parent.clone()));
        assert_eq!(child2.borrow().as_node().parent(), Some(parent.clone()));
    }
}
