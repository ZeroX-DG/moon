use super::comment::Comment;
use super::document::Document;
use super::dom_ref::{NodeRef, WeakNodeRef};
use super::element::Element;
use super::elements::ElementData;
use super::node_list::NodeList;
use super::text::Text;
use enum_dispatch::enum_dispatch;

pub struct Node {
    parent_node: Option<WeakNodeRef>,
    first_child: Option<NodeRef>,
    last_child: Option<WeakNodeRef>,
    next_sibling: Option<NodeRef>,
    prev_sibling: Option<WeakNodeRef>,
    owner_document: Option<WeakNodeRef>,
    data: Option<NodeData>,
}

#[enum_dispatch(NodeHooks)]
pub enum NodeData {
    Element(Element),
    Text(Text),
    Document(Document),
    Comment(Comment),
}

#[enum_dispatch]
pub trait NodeHooks {
    #[allow(unused_variables)]
    fn on_inserted(&mut self, document: NodeRef) {
    }
}

impl core::fmt::Debug for Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Node({:?})", self.data)
    }
}

impl NodeData {
    pub fn handle_on_inserted(&mut self, document: NodeRef) {
        self.on_inserted(document);
    }
}

impl core::fmt::Debug for NodeData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NodeData::Text(text) => write!(f, "Text({:?})", text.get_data()),
            NodeData::Comment(comment) => write!(f, "Comment({:?})", comment.get_data()),
            NodeData::Document(_) => write!(f, "Document"),
            NodeData::Element(element) => write!(f, "Element({:?})", element.tag_name()),
        }
    }
}

impl Node {
    pub fn new(data: NodeData) -> Self {
        let mut node = Self::empty();
        node.data = Some(data);
        node
    }

    pub fn empty() -> Self {
        Self {
            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            owner_document: None,
            data: None,
        }
    }

    /// Set the owner document for node
    pub fn set_document(&mut self, doc: WeakNodeRef) {
        self.owner_document = Some(doc);
    }

    /// Children list
    pub fn child_nodes(&self) -> NodeList {
        NodeList::new(self.first_child())
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

    /// Descendant text content of the node
    /// https://dom.spec.whatwg.org/#concept-descendant-text-content
    pub fn descendant_text_content(&self) -> String {
        if let Some(text) = self.as_text_opt() {
            return text.get_data();
        }
        let mut result = String::new();
        for child in self.child_nodes() {
            result.push_str(&child.borrow().descendant_text_content());
        }
        result
    }

    /// Child text content of the node
    /// https://dom.spec.whatwg.org/#concept-child-text-content
    pub fn child_text_content(&self) -> String {
        let mut result = String::new();
        for child in self.child_nodes() {
            if let Some(text) = child.borrow().as_text_opt() {
                result.push_str(&text.get_data());
            }
        }
        result
    }

    /// Detach node from the parent
    pub fn detach(node_ref: &NodeRef) {
        let mut node = node_ref.borrow_mut();

        let parent = node.parent();

        if let Some(parent) = parent {
            let mut parent_node = parent.borrow_mut();
            {
                let first_child = parent_node.first_child().unwrap();
                let last_child = parent_node.last_child().unwrap();

                if *node_ref == first_child {
                    parent_node.first_child = node.next_sibling();
                } else if *node_ref == last_child {
                    parent_node.last_child = node.prev_sibling.clone();
                }
            }

            let mut child = parent_node.first_child();

            while let Some(c) = child {
                if c == *node_ref {
                    let inner = c.borrow_mut();
                    let previous_sibling = inner.prev_sibling();
                    let next_sibling = inner.next_sibling();

                    if let Some(prev) = previous_sibling {
                        if let Some(next) = next_sibling {
                            prev.borrow_mut().next_sibling = Some(next.clone());
                            next.borrow_mut().prev_sibling = Some(prev.downgrade());
                        } else {
                            prev.borrow_mut().next_sibling = None;
                        }
                    } else {
                        if let Some(next) = next_sibling {
                            next.borrow_mut().prev_sibling = None;
                        }
                    }
                    break;
                } else {
                    child = c.borrow().next_sibling();
                }
            }
        }

        node.parent_node = None;
        node.prev_sibling = None;
        node.next_sibling = None;
    }

    /// Transfer parent of nodes
    pub fn reparent_nodes_in_node(old_parent: NodeRef, new_parent: NodeRef) {
        let mut old = old_parent.borrow_mut();
        let mut new = new_parent.borrow_mut();
        new.first_child = old.first_child.take();
        new.last_child = old.last_child.take();

        for child in new.child_nodes() {
            child.borrow_mut().parent_node = Some(new_parent.clone().downgrade());
        }
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
        // detach from parent
        Node::detach(&child);

        let mut parent_node = parent.borrow_mut();

        let mut child_node = child.borrow_mut();

        if let Some(last_child) = parent_node.last_child() {
            last_child.borrow_mut().next_sibling = Some(child.clone());
            child_node.prev_sibling = Some(last_child.clone().downgrade());
        }

        child_node.parent_node = Some(parent.clone().downgrade());

        if parent_node.first_child().is_none() {
            parent_node.first_child = Some(child.clone());
        }

        parent_node.last_child = Some(child.clone().downgrade());
        let document = child_node.owner_document().clone().unwrap();
        if let Some(data) = &mut child_node.data {
            data.handle_on_inserted(document);
        }
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
        Node::detach(&child);
        if let Some(ref_child) = ref_child {
            let mut ref_child_node = ref_child.borrow_mut();

            let mut parent_node = parent.borrow_mut();

            let mut child_node = child.borrow_mut();

            // set parent of inserted node to be this parent
            child_node.parent_node = Some(parent.clone().downgrade());

            // if the reference child has previous sibling (not first child)
            // the inserted child will become reference child previous sibling
            if let Some(prev_sibling) = ref_child_node.prev_sibling() {
                prev_sibling.borrow_mut().next_sibling = Some(child.clone());
                child_node.prev_sibling = Some(prev_sibling.clone().downgrade());
            } else {
                // otherwise this is the first child of parent
                // update first child
                parent_node.first_child = Some(child.clone());
            }

            // set inserted child to be new previous sibling of ref child
            ref_child_node.prev_sibling = Some(child.clone().downgrade());
            child_node.next_sibling = Some(ref_child.clone());
        } else {
            Node::append_child(parent, child);
        }
    }
}

impl Node {
    pub fn as_text_opt(&self) -> Option<&Text> {
        match &self.data {
            Some(NodeData::Text(text)) => Some(text),
            _ => None,
        }
    }

    pub fn as_text_mut_opt(&mut self) -> Option<&mut Text> {
        match &mut self.data {
            Some(NodeData::Text(text)) => Some(text),
            _ => None,
        }
    }

    pub fn as_element_opt(&self) -> Option<&Element> {
        match &self.data {
            Some(NodeData::Element(element)) => Some(element),
            _ => None,
        }
    }

    pub fn as_element_mut_opt(&mut self) -> Option<&mut Element> {
        match &mut self.data {
            Some(NodeData::Element(element)) => Some(element),
            _ => None,
        }
    }

    pub fn as_document_opt(&self) -> Option<&Document> {
        match &self.data {
            Some(NodeData::Document(doc)) => Some(doc),
            _ => None,
        }
    }

    pub fn as_document_mut_opt(&mut self) -> Option<&mut Document> {
        match &mut self.data {
            Some(NodeData::Document(doc)) => Some(doc),
            _ => None,
        }
    }

    pub fn as_comment_opt(&self) -> Option<&Comment> {
        match &self.data {
            Some(NodeData::Comment(com)) => Some(com),
            _ => None,
        }
    }

    pub fn as_comment_mut_opt(&mut self) -> Option<&mut Comment> {
        match &mut self.data {
            Some(NodeData::Comment(com)) => Some(com),
            _ => None,
        }
    }

    pub fn as_element(&self) -> &Element {
        self.as_element_opt().expect("Node is not an Element")
    }

    pub fn as_element_mut(&mut self) -> &mut Element {
        self.as_element_mut_opt().expect("Node is not an Element")
    }

    pub fn as_document(&self) -> &Document {
        self.as_document_opt().expect("Node is not a Document")
    }

    pub fn as_document_mut(&mut self) -> &mut Document {
        self.as_document_mut_opt().expect("Node is not a Document")
    }

    pub fn as_comment(&self) -> &Comment {
        self.as_comment_opt().expect("Node is not a Comment")
    }

    pub fn as_comment_mut(&mut self) -> &mut Comment {
        self.as_comment_mut_opt().expect("Node is not a Comment")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn append_child_first_child() {
        let parent = NodeRef::new(Node::empty());
        let child = NodeRef::new(Node::empty());

        Node::append_child(parent.clone(), child.clone());

        assert_eq!(parent.borrow().first_child(), Some(child.clone()));
        assert_eq!(parent.borrow().last_child(), Some(child.clone()));
        assert_eq!(child.borrow().parent(), Some(parent.clone()));
        assert_eq!(child.borrow().prev_sibling(), None);
        assert_eq!(child.borrow().next_sibling(), None);
    }

    #[test]
    fn append_child_normal() {
        let parent = NodeRef::new(Node::empty());
        let child1 = NodeRef::new(Node::empty());
        let child2 = NodeRef::new(Node::empty());

        Node::append_child(parent.clone(), child1.clone());
        Node::append_child(parent.clone(), child2.clone());

        assert_eq!(parent.borrow().first_child(), Some(child1.clone()));
        assert_eq!(parent.borrow().last_child(), Some(child2.clone()));
        assert_eq!(child1.borrow().next_sibling(), Some(child2.clone()));
        assert_eq!(child2.borrow().prev_sibling(), Some(child1.clone()));
        assert_eq!(child1.borrow().prev_sibling(), None);
        assert_eq!(child2.borrow().next_sibling(), None);

        assert_eq!(child1.borrow().parent(), Some(parent.clone()));
        assert_eq!(child2.borrow().parent(), Some(parent.clone()));
    }

    #[test]
    fn insert_before_normal() {
        let parent = NodeRef::new(Node::empty());
        let child1 = NodeRef::new(Node::empty());
        let child2 = NodeRef::new(Node::empty());

        Node::append_child(parent.clone(), child1.clone());
        Node::insert_before(parent.clone(), child2.clone(), Some(child1.clone()));

        assert_eq!(parent.borrow().first_child(), Some(child2.clone()));
        assert_eq!(parent.borrow().last_child(), Some(child1.clone()));
        assert_eq!(child2.borrow().next_sibling(), Some(child1.clone()));
        assert_eq!(child1.borrow().prev_sibling(), Some(child2.clone()));
        assert_eq!(child2.borrow().prev_sibling(), None);
        assert_eq!(child1.borrow().next_sibling(), None);

        assert_eq!(child1.borrow().parent(), Some(parent.clone()));
        assert_eq!(child2.borrow().parent(), Some(parent.clone()));
    }

    #[test]
    fn insert_before_empty_ref_node() {
        let parent = NodeRef::new(Node::empty());
        let child = NodeRef::new(Node::empty());

        Node::insert_before(parent.clone(), child.clone(), None);

        assert_eq!(parent.borrow().first_child(), Some(child.clone()));
        assert_eq!(parent.borrow().last_child(), Some(child.clone()));
        assert_eq!(child.borrow().parent(), Some(parent.clone()));
        assert_eq!(child.borrow().prev_sibling(), None);
        assert_eq!(child.borrow().next_sibling(), None);
    }

    #[test]
    fn detach_before_append() {
        let parent = NodeRef::new(Node::empty());
        let child = NodeRef::new(Node::empty());

        Node::append_child(parent.clone(), child.clone());

        assert_eq!(parent.borrow().first_child(), Some(child.clone()));
        assert_eq!(child.borrow().parent(), Some(parent.clone()));

        let new_parent = NodeRef::new(Node::empty());

        Node::append_child(new_parent.clone(), child.clone());

        assert_eq!(parent.borrow().first_child(), None);
        assert_eq!(new_parent.borrow().first_child(), Some(child.clone()));
        assert_eq!(child.borrow().parent(), Some(new_parent.clone()));
    }
}
