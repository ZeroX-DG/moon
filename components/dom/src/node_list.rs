use super::dom_ref::NodeRef;

#[derive(Debug)]
pub struct NodeList {
    start: Option<NodeRef>,
}

impl NodeList {
    pub fn new(start: Option<NodeRef>) -> Self {
        Self { start }
    }

    pub fn item(&self, index: usize) -> Option<NodeRef> {
        let mut node = self.start.clone();
        let mut current_idx = index;
        while let Some(node_ref) = &node {
            if current_idx == 0 {
                break;
            }
            node = node_ref.clone().borrow().as_node().next_sibling();
            current_idx -= 1;
        }
        node
    }

    pub fn length(&self) -> usize {
        let mut node = self.start.clone();
        let mut length = 0;
        while let Some(node_ref) = &node {
            node = node_ref.clone().borrow().as_node().next_sibling();
            length += 1;
        }
        length
    }
}

pub struct NodeListIterator {
    node_list: NodeList,
    index: usize,
}

impl Iterator for NodeListIterator {
    type Item = NodeRef;

    fn next(&mut self) -> Option<NodeRef> {
        let result = self.node_list.item(self.index);
        self.index += 1;
        result
    }
}

impl IntoIterator for NodeList {
    type Item = NodeRef;
    type IntoIter = NodeListIterator;

    fn into_iter(self) -> Self::IntoIter {
        NodeListIterator {
            node_list: self,
            index: 0,
        }
    }
}
