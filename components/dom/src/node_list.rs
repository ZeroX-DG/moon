use super::node::NodeRef;
use super::implementations::Node;

pub struct NodeList<'a> {
    start: Option<NodeRef<'a>>
}

impl<'a> NodeList<'a> {
    pub fn new(start: Option<NodeRef<'a>>) -> Self {
        Self {
            start
        }
    }

    pub fn item(&self, index: usize) -> Option<NodeRef<'a>> {
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
