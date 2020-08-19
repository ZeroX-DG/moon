use super::node::NodeRef;

pub struct NodeList {
    start: Option<NodeRef>
}

impl NodeList {
    pub fn new(start: Option<NodeRef>) -> Self {
        Self {
            start
        }
    }

    pub fn item(&self, index: usize) -> Option<NodeRef> {
        let mut node = self.start.clone();
        let mut current_idx = index;
        while let Some(node_ref) = &node {
            if current_idx == 0 {
                break
            }
            node = node_ref.next_sibling();
            current_idx -= 1;
        }
        node
    }

    pub fn length(&self) -> usize {
        let mut node = self.start.clone();
        let mut length = 0;
        while let Some(node_ref) = &node {
            node = node_ref.next_sibling();
            length += 1;
        }
        length
    }
}
