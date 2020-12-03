use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug)]
pub struct Tree<T> {
    arena: HashMap<usize, TreeNode<T>>,
    next_id: usize
}

#[derive(Debug)]
pub struct TreeNode<T> {
    id: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    inner: T
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self {
            arena: HashMap::new(),
            next_id: 0
        }
    }

    pub fn allocate(&mut self, node: TreeNode<T>) {
        self.arena.insert(node.id, node);
        self.next_id += 1;
    }

    pub fn new_node(&mut self, inner: T) -> usize {
        let node_id = self.next_id;
        let node = TreeNode {
            id: node_id,
            parent: None,
            children: Vec::new(),
            inner
        };

        self.allocate(node);
        node_id
    }

    pub fn get_node(&mut self, node_id: usize) -> Option<&TreeNode<T>> {
        self.arena.get(&node_id)
    }
}

impl<T> Deref for TreeNode<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
