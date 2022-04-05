use std::{rc::{Rc, Weak}, cell::RefCell, ops::Deref, fmt::Debug};

pub struct TreeNode<T: TreeNodeHooks<T> + Debug>(Rc<Node<T>>);
pub struct WeakTreeNode<T: TreeNodeHooks<T> + Debug>(Weak<Node<T>>);

pub type NullableWeakNode<T> = Option<WeakTreeNode<T>>;
pub type NullableNode<T> = Option<TreeNode<T>>;

#[allow(unused_variables)]
pub trait TreeNodeHooks<T: TreeNodeHooks<T> + Debug> {
    fn on_inserted(&self, current: TreeNode<T>, parent: TreeNode<T>) {}
    fn on_children_updated(&self, current: TreeNode<T>) {}
}

pub struct Node<T: TreeNodeHooks<T> + Debug> {
    data: T,
    parent_node: RefCell<NullableWeakNode<T>>,
    first_child: RefCell<NullableNode<T>>,
    last_child: RefCell<NullableNode<T>>,
    next_sibling: RefCell<NullableNode<T>>,
    prev_sibling: RefCell<NullableWeakNode<T>>,
}

impl<T: TreeNodeHooks<T> + Debug> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            parent_node: RefCell::new(None),
            first_child: RefCell::new(None),
            last_child: RefCell::new(None),
            next_sibling: RefCell::new(None),
            prev_sibling: RefCell::new(None),
            data,
        }
    }
}

impl<T: TreeNodeHooks<T> + Debug> TreeNode<T> {
    pub fn new(data: T) -> Self {
        Self(Rc::new(Node::new(data)))
    }

    /// First child of the node
    pub fn first_child(&self) -> NullableNode<T> {
        self.first_child.borrow().clone()
    }

    /// Last child of the node
    pub fn last_child(&self) -> NullableNode<T> {
        self.last_child.borrow().clone()
    }

    /// Next sibling of the node
    pub fn next_sibling(&self) -> NullableNode<T> {
        self.next_sibling.borrow().clone()
    }

    /// Previous sibling of the node
    pub fn prev_sibling(&self) -> NullableNode<T> {
        match self.0.prev_sibling.borrow().deref() {
            Some(node) => node.upgrade().map(|rc| TreeNode::from(rc)),
            _ => None,
        }
    }

    /// Parent of the node
    pub fn parent(&self) -> NullableNode<T> {
        match self.0.parent_node.borrow().deref() {
            Some(node) => node.upgrade().map(|rc| TreeNode::from(rc)),
            _ => None,
        }
    }

    /// Detach node from the parent
    pub fn detach(&self) {
        if let Some(previous_sibling) = self.prev_sibling() {
            previous_sibling.next_sibling.replace(self.next_sibling());
        }
        if let Some(next_sibling) = self.next_sibling() {
            next_sibling
                .prev_sibling
                .replace(self.prev_sibling.borrow().clone());
        }
        if let Some(parent) = self.parent() {
            let first_child = parent.first_child().unwrap();
            let last_child = parent.last_child().unwrap();

            if Rc::ptr_eq(&self, &first_child) {
                parent.first_child.replace(self.next_sibling());
            } else if Rc::ptr_eq(&self, &last_child) {
                parent.last_child.replace(self.prev_sibling());
            }
        }

        self.parent_node.replace(None);
        self.prev_sibling.replace(None);
        self.next_sibling.replace(None);
    }

    pub fn for_each_child<F>(&self, mut callback: F)
    where
        F: FnMut(TreeNode<T>)
    {
        let mut maybe_child = self.first_child();
        while let Some(child) = maybe_child  {
            callback(child.clone());
            maybe_child = child.next_sibling();
        }
    }

    /// Transfer parent of nodes
    pub fn transfer_children_to_node(&self, new_parent: TreeNode<T>) {
        new_parent.for_each_child(|child| child.detach());
        new_parent.first_child.replace(self.first_child());
        new_parent.last_child.replace(self.last_child());

        new_parent.for_each_child(|child| {
            child.parent_node.replace(Some(WeakTreeNode::from(new_parent.clone())));
        });
    }

    /// Append a child node to a parent node
    ///
    /// **Ensure that:**
    /// 1. Last child of the parent is the child
    /// 2. First child of the parent is also the child if the parent has only 1 child
    /// 3. The child parent is this parent
    /// 4. The next-to-last child of the parent next sibling is the child if the parent has more
    ///    than 1 child
    pub fn append_child(&self, child: TreeNode<T>) {
        // detach from parent
        child.detach();

        if let Some(last_child) = self.last_child() {
            last_child.next_sibling.replace(Some(child.clone()));
            child.prev_sibling.replace(Some(WeakTreeNode::from(last_child)));
        }

        child.parent_node.replace(Some(WeakTreeNode::from(self)));

        if self.first_child().is_none() {
            self.first_child.replace(Some(child.clone()));
        }

        self.last_child.replace(Some(child.clone()));

        // trigger hook callback
        child.data.on_inserted(child.clone(), self.clone());
        self.data.on_children_updated(self.clone());
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
    pub fn insert_before(&self, child: TreeNode<T>, ref_child: Option<TreeNode<T>>) {
        child.detach();
        if let Some(ref_child) = ref_child {
            // set parent of inserted node to be this parent
            child.parent_node.replace(Some(WeakTreeNode::from(self)));

            // if the reference child has previous sibling (not first child)
            // the inserted child will become reference child previous sibling
            if let Some(prev_sibling) = ref_child.prev_sibling() {
                prev_sibling.next_sibling.replace(Some(child.clone()));
                child
                    .prev_sibling
                    .replace(Some(WeakTreeNode::from(prev_sibling)));
            } else {
                // otherwise this is the first child of parent
                // update first child
                self.first_child.replace(Some(child.clone()));
            }

            // set inserted child to be new previous sibling of ref child
            ref_child.prev_sibling.replace(Some(WeakTreeNode::from(child.clone())));
            child.next_sibling.replace(Some(ref_child));
        } else {
            self.append_child(child);
        }
    }
}

impl<T: TreeNodeHooks<T> + Debug> WeakTreeNode<T> {
    pub fn upgrade(&self) -> Option<TreeNode<T>> {
        self.0.upgrade().map(|rc| TreeNode::from(rc))
    }
}

impl<T: TreeNodeHooks<T> + Debug> Deref for Node<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: TreeNodeHooks<T> + Debug> Deref for TreeNode<T> {
    type Target = Rc<Node<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: TreeNodeHooks<T> + Debug> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        TreeNode(self.0.clone())
    }
}

impl<T: TreeNodeHooks<T> + Debug> Deref for WeakTreeNode<T> {
    type Target = Weak<Node<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: TreeNodeHooks<T> + Debug> Clone for WeakTreeNode<T> {
    fn clone(&self) -> Self {
        WeakTreeNode(self.0.clone())
    }
}

impl<T: TreeNodeHooks<T> + Debug> From<Rc<Node<T>>> for TreeNode<T> {
    fn from(rc: Rc<Node<T>>) -> Self {
        TreeNode(rc)
    }
}

impl<T: TreeNodeHooks<T> + Debug> Debug for TreeNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl<T: TreeNodeHooks<T> + Debug> From<TreeNode<T>> for WeakTreeNode<T> {
    fn from(rc: TreeNode<T>) -> Self {
        WeakTreeNode(Rc::downgrade(&rc))
    }
}

impl<T: TreeNodeHooks<T> + Debug> From<&TreeNode<T>> for WeakTreeNode<T> {
    fn from(rc: &TreeNode<T>) -> Self {
        WeakTreeNode(Rc::downgrade(rc))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    pub struct TestNode;
    impl TreeNodeHooks<TestNode> for TestNode {}

    fn assert_node_eq(a: NullableNode<TestNode>, b: NullableNode<TestNode>) {
        let result = match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
            _ => false,
        };

        assert!(result)
    }

    #[test]
    fn append_child_first_child() {
        let parent = TreeNode::new(TestNode);
        let child = TreeNode::new(TestNode);

        parent.append_child(child.clone());

        assert_node_eq(parent.first_child(), Some(child.clone()));
        assert_node_eq(parent.last_child(), Some(child.clone()));
        assert_node_eq(child.parent(), Some(parent.clone()));
        assert_node_eq(child.prev_sibling(), None);
        assert_node_eq(child.next_sibling(), None);
    }

    #[test]
    fn append_child_normal() {
        let parent = TreeNode::new(TestNode);
        let child1 = TreeNode::new(TestNode);
        let child2 = TreeNode::new(TestNode);

        parent.append_child(child1.clone());
        parent.append_child(child2.clone());

        assert_node_eq(parent.first_child(), Some(child1.clone()));
        assert_node_eq(parent.last_child(), Some(child2.clone()));
        assert_node_eq(child1.next_sibling(), Some(child2.clone()));
        assert_node_eq(child2.prev_sibling(), Some(child1.clone()));
        assert_node_eq(child1.prev_sibling(), None);
        assert_node_eq(child2.next_sibling(), None);

        assert_node_eq(child1.parent(), Some(parent.clone()));
        assert_node_eq(child2.parent(), Some(parent.clone()));
    }

    #[test]
    fn insert_before_normal() {
        let parent = TreeNode::new(TestNode);
        let child1 = TreeNode::new(TestNode);
        let child2 = TreeNode::new(TestNode);

        parent.append_child(child1.clone());
        parent.insert_before(child2.clone(), Some(child1.clone()));

        assert_node_eq(parent.first_child(), Some(child2.clone()));
        assert_node_eq(parent.last_child(), Some(child1.clone()));
        assert_node_eq(child2.next_sibling(), Some(child1.clone()));
        assert_node_eq(child1.prev_sibling(), Some(child2.clone()));
        assert_node_eq(child2.prev_sibling(), None);
        assert_node_eq(child1.next_sibling(), None);

        assert_node_eq(child1.parent(), Some(parent.clone()));
        assert_node_eq(child2.parent(), Some(parent.clone()));
    }

    #[test]
    fn insert_before_empty_ref_node() {
        let parent = TreeNode::new(TestNode);
        let child = TreeNode::new(TestNode);

        parent.insert_before(child.clone(), None);

        assert_node_eq(parent.first_child(), Some(child.clone()));
        assert_node_eq(parent.last_child(), Some(child.clone()));
        assert_node_eq(child.parent(), Some(parent.clone()));
        assert_node_eq(child.prev_sibling(), None);
        assert_node_eq(child.next_sibling(), None);
    }

    #[test]
    fn detach_before_append() {
        let parent = TreeNode::new(TestNode);
        let child = TreeNode::new(TestNode);

        parent.append_child(child.clone());

        assert_node_eq(parent.first_child(), Some(child.clone()));
        assert_node_eq(child.parent(), Some(parent.clone()));

        let new_parent = TreeNode::new(TestNode);

        new_parent.append_child(child.clone());

        assert_node_eq(parent.first_child(), None);
        assert_node_eq(new_parent.first_child(), Some(child.clone()));
        assert_node_eq(child.parent(), Some(new_parent.clone()));
    }
}
