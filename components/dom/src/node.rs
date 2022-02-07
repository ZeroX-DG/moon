use super::comment::Comment;
use super::document::Document;
use super::element::Element;
use super::elements::ElementData;
use super::node_list::NodeList;
use super::text::Text;
use enum_dispatch::enum_dispatch;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

pub type NullableWeakNode = Option<Weak<Node>>;
pub type NullableNode = Option<Rc<Node>>;

pub struct Node {
    parent_node: RefCell<NullableWeakNode>,
    first_child: RefCell<NullableNode>,
    last_child: RefCell<NullableNode>,
    next_sibling: RefCell<NullableNode>,
    prev_sibling: RefCell<NullableWeakNode>,
    owner_document: RefCell<NullableWeakNode>,
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
    fn on_inserted(&self, document: Rc<Node>) {}
}

impl core::fmt::Debug for Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl NodeData {
    pub fn handle_on_inserted(&self, document: Rc<Node>) {
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
            parent_node: RefCell::new(None),
            first_child: RefCell::new(None),
            last_child: RefCell::new(None),
            next_sibling: RefCell::new(None),
            prev_sibling: RefCell::new(None),
            owner_document: RefCell::new(None),
            data: None,
        }
    }

    /// Set the owner document for node
    pub fn set_document(&self, doc: Weak<Node>) {
        self.owner_document.replace(Some(doc));
    }

    /// Children list
    pub fn child_nodes(&self) -> NodeList {
        NodeList::new(self.first_child())
    }

    /// First child of the node
    pub fn first_child(&self) -> NullableNode {
        self.first_child.borrow().clone()
    }

    /// Last child of the node
    pub fn last_child(&self) -> NullableNode {
        self.last_child.borrow().clone()
    }

    /// Next sibling of the node
    pub fn next_sibling(&self) -> NullableNode {
        self.next_sibling.borrow().clone()
    }

    /// Previous sibling of the node
    pub fn prev_sibling(&self) -> NullableNode {
        match self.prev_sibling.borrow().deref() {
            Some(node) => node.upgrade(),
            _ => None,
        }
    }

    /// Parent of the node
    pub fn parent(&self) -> NullableNode {
        match self.parent_node.borrow().deref() {
            Some(node) => node.upgrade(),
            _ => None,
        }
    }

    /// Owner document of the node
    pub fn owner_document(&self) -> NullableNode {
        match self.owner_document.borrow().deref() {
            Some(node) => node.upgrade(),
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
            result.push_str(&child.descendant_text_content());
        }
        result
    }

    /// Child text content of the node
    /// https://dom.spec.whatwg.org/#concept-child-text-content
    pub fn child_text_content(&self) -> String {
        let mut result = String::new();
        for child in self.child_nodes() {
            if let Some(text) = child.as_text_opt() {
                result.push_str(&text.get_data());
            }
        }
        result
    }

    /// Detach node from the parent
    pub fn detach(node: Rc<Node>) {
        if let Some(previous_sibling) = node.prev_sibling() {
            previous_sibling.next_sibling.replace(node.next_sibling());
        }
        if let Some(next_sibling) = node.next_sibling() {
            next_sibling
                .prev_sibling
                .replace(node.prev_sibling.borrow().clone());
        }
        if let Some(parent) = node.parent() {
            let first_child = parent.first_child().unwrap();
            let last_child = parent.last_child().unwrap();

            if Rc::ptr_eq(&node, &first_child) {
                parent.first_child.replace(node.next_sibling());
            } else if Rc::ptr_eq(&node, &last_child) {
                parent.last_child.replace(node.prev_sibling());
            }
        }

        node.parent_node.replace(None);
        node.prev_sibling.replace(None);
        node.next_sibling.replace(None);
    }

    /// Transfer parent of nodes
    pub fn reparent_nodes_in_node(old_parent: Rc<Node>, new_parent: Rc<Node>) {
        for child in new_parent.child_nodes() {
            Node::detach(child);
        }
        new_parent.first_child.replace(old_parent.first_child());
        new_parent.last_child.replace(old_parent.last_child());

        for child in new_parent.child_nodes() {
            child.parent_node.replace(Some(Rc::downgrade(&new_parent)));
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
    pub fn append_child(parent: Rc<Node>, child: Rc<Node>) {
        // detach from parent
        Node::detach(child.clone());

        if let Some(last_child) = parent.last_child() {
            last_child.next_sibling.replace(Some(child.clone()));
            child.prev_sibling.replace(Some(Rc::downgrade(&last_child)));
        }

        child.parent_node.replace(Some(Rc::downgrade(&parent)));

        if parent.first_child().is_none() {
            parent.first_child.replace(Some(child.clone()));
        }

        parent.last_child.replace(Some(child.clone()));
        let document = child.owner_document().clone().unwrap();
        if let Some(data) = &child.data {
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
    pub fn insert_before(parent: Rc<Node>, child: Rc<Node>, ref_child: Option<Rc<Node>>) {
        Node::detach(child.clone());
        if let Some(ref_child) = ref_child {
            // set parent of inserted node to be this parent
            child.parent_node.replace(Some(Rc::downgrade(&parent)));

            // if the reference child has previous sibling (not first child)
            // the inserted child will become reference child previous sibling
            if let Some(prev_sibling) = ref_child.prev_sibling() {
                prev_sibling.next_sibling.replace(Some(child.clone()));
                child
                    .prev_sibling
                    .replace(Some(Rc::downgrade(&prev_sibling)));
            } else {
                // otherwise this is the first child of parent
                // update first child
                parent.first_child.replace(Some(child.clone()));
            }

            // set inserted child to be new previous sibling of ref child
            ref_child.prev_sibling.replace(Some(Rc::downgrade(&child)));
            child.next_sibling.replace(Some(ref_child.clone()));
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

    pub fn as_element_opt(&self) -> Option<&Element> {
        match &self.data {
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

    pub fn as_comment_opt(&self) -> Option<&Comment> {
        match &self.data {
            Some(NodeData::Comment(com)) => Some(com),
            _ => None,
        }
    }

    pub fn as_element(&self) -> &Element {
        self.as_element_opt().expect("Node is not an Element")
    }

    pub fn as_document(&self) -> &Document {
        self.as_document_opt().expect("Node is not a Document")
    }

    pub fn as_comment(&self) -> &Comment {
        self.as_comment_opt().expect("Node is not a Comment")
    }

    pub fn as_text(&self) -> &Text {
        self.as_text_opt().expect("Node is not a Text")
    }

    pub fn is_element(&self) -> bool {
        self.as_element_opt().is_some()
    }

    pub fn is_document(&self) -> bool {
        self.as_document_opt().is_some()
    }

    pub fn is_comment(&self) -> bool {
        self.as_comment_opt().is_some()
    }

    pub fn is_text(&self) -> bool {
        self.as_text_opt().is_some()
    }

    pub fn data(&self) -> &Option<NodeData> {
        &self.data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    pub fn assert_node_eq(a: NullableNode, b: NullableNode) {
        let result = match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
            _ => false,
        };

        assert!(result)
    }

    #[test]
    fn append_child_first_child() {
        let parent = Rc::new(Node::empty());
        let child = Rc::new(Node::empty());

        let doc = Rc::new(Node::new(NodeData::Document(Document::new())));

        parent.set_document(Rc::downgrade(&doc));
        child.set_document(Rc::downgrade(&doc));

        Node::append_child(parent.clone(), child.clone());

        assert_node_eq(parent.first_child(), Some(child.clone()));
        assert_node_eq(parent.last_child(), Some(child.clone()));
        assert_node_eq(child.parent(), Some(parent.clone()));
        assert_node_eq(child.prev_sibling(), None);
        assert_node_eq(child.next_sibling(), None);
    }

    #[test]
    fn append_child_normal() {
        let parent = Rc::new(Node::empty());
        let child1 = Rc::new(Node::empty());
        let child2 = Rc::new(Node::empty());

        let doc = Rc::new(Node::new(NodeData::Document(Document::new())));

        parent.set_document(Rc::downgrade(&doc));
        child1.set_document(Rc::downgrade(&doc));
        child2.set_document(Rc::downgrade(&doc));

        Node::append_child(parent.clone(), child1.clone());
        Node::append_child(parent.clone(), child2.clone());

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
        let parent = Rc::new(Node::empty());
        let child1 = Rc::new(Node::empty());
        let child2 = Rc::new(Node::empty());

        let doc = Rc::new(Node::new(NodeData::Document(Document::new())));

        parent.set_document(Rc::downgrade(&doc));
        child1.set_document(Rc::downgrade(&doc));
        child2.set_document(Rc::downgrade(&doc));

        Node::append_child(parent.clone(), child1.clone());
        Node::insert_before(parent.clone(), child2.clone(), Some(child1.clone()));

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
        let parent = Rc::new(Node::empty());
        let child = Rc::new(Node::empty());

        let doc = Rc::new(Node::new(NodeData::Document(Document::new())));

        parent.set_document(Rc::downgrade(&doc));
        child.set_document(Rc::downgrade(&doc));

        Node::insert_before(parent.clone(), child.clone(), None);

        assert_node_eq(parent.first_child(), Some(child.clone()));
        assert_node_eq(parent.last_child(), Some(child.clone()));
        assert_node_eq(child.parent(), Some(parent.clone()));
        assert_node_eq(child.prev_sibling(), None);
        assert_node_eq(child.next_sibling(), None);
    }

    #[test]
    fn detach_before_append() {
        let parent = Rc::new(Node::empty());
        let child = Rc::new(Node::empty());

        let doc = Rc::new(Node::new(NodeData::Document(Document::new())));

        parent.set_document(Rc::downgrade(&doc));
        child.set_document(Rc::downgrade(&doc));

        Node::append_child(parent.clone(), child.clone());

        assert_node_eq(parent.first_child(), Some(child.clone()));
        assert_node_eq(child.parent(), Some(parent.clone()));

        let new_parent = Rc::new(Node::empty());

        Node::append_child(new_parent.clone(), child.clone());

        assert_node_eq(parent.first_child(), None);
        assert_node_eq(new_parent.first_child(), Some(child.clone()));
        assert_node_eq(child.parent(), Some(new_parent.clone()));
    }
}
