use std::collections::HashMap;

pub type TreeNodeId = usize;

pub struct Tree<T> {
    arena: HashMap<TreeNodeId, TreeNode<T>>,
    root: Option<TreeNodeId>,
    next_id: TreeNodeId,
}

pub struct TreeNode<T> {
    id: TreeNodeId,
    data: T,
    children: Vec<TreeNodeId>,
    parent: Option<TreeNodeId>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self {
            arena: HashMap::new(),
            next_id: 0,
            root: None,
        }
    }

    pub fn add_child(&mut self, parent_id: &TreeNodeId, child_data: T) -> TreeNodeId {
        let child_id = self.save_node(child_data);

        self.add_child_by_id(parent_id, &child_id);

        child_id
    }

    pub fn add_child_by_id(&mut self, parent_id: &TreeNodeId, child_id: &TreeNodeId) {
        self.get_node_mut(parent_id).children.push(*child_id);
        self.get_node_mut(child_id).parent = Some(parent_id.clone());
    }

    pub fn get_node_mut(&mut self, node_id: &TreeNodeId) -> &mut TreeNode<T> {
        self.arena.get_mut(node_id).unwrap()
    }

    pub fn get_node(&self, node_id: &TreeNodeId) -> &TreeNode<T> {
        self.arena.get(node_id).unwrap()
    }

    pub fn root(&self) -> Option<TreeNodeId> {
        self.root
    }

    pub fn set_root(&mut self, node_data: T) -> TreeNodeId {
        let node_id = self.save_node(node_data);
        self.root = Some(node_id);
        node_id
    }

    pub fn children_mut(&mut self, node_id: &TreeNodeId) -> &mut Vec<TreeNodeId> {
        &mut self.get_node_mut(node_id).children
    }

    pub fn children(&self, node_id: &TreeNodeId) -> &[TreeNodeId] {
        &self.get_node(node_id).children
    }

    pub fn parent(&self, node_id: &TreeNodeId) -> Option<&TreeNode<T>> {
        self.get_node(node_id)
            .parent()
            .map(|parent_id| self.get_node(&parent_id))
    }

    fn save_node(&mut self, node_data: T) -> TreeNodeId {
        let node_id = self.request_id();
        let node = TreeNode::new(node_id, node_data);
        self.arena.insert(node_id, node);
        node_id
    }

    fn request_id(&mut self) -> TreeNodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl<T> TreeNode<T> {
    pub fn new(id: TreeNodeId, data: T) -> Self {
        Self {
            id: id,
            data: data,
            children: Vec::new(),
            parent: None,
        }
    }

    pub fn id(&self) -> TreeNodeId {
        self.id
    }

    pub fn set_children(&mut self, children: Vec<TreeNodeId>) {
        self.children = children;
    }

    pub fn parent(&self) -> Option<TreeNodeId> {
        self.parent
    }
}

impl<T> std::ops::Deref for TreeNode<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for TreeNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
