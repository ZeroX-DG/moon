use std::rc::Weak;

pub struct Node {
    parent: Weak<Node>,
    child_nodes: Vec<Node>
}

impl Node {
    pub fn new() -> Self {
        Self {
            parent: Weak::new(),
            child_nodes: Vec::new()
        }
    }

    pub fn first_child(&self) -> Option<&Node> {
        self.child_nodes.first()
    }

    pub fn last_child(&self) -> Option<&Node> {
        self.child_nodes.last()
    }
}
