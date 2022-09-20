use crate::node_list::NodeList;

use super::comment::Comment;
use super::document::Document;
use super::element::Element;
use super::elements::ElementData;
use super::text::Text;
use enum_dispatch::enum_dispatch;
use shared::tree_node::{TreeNode, TreeNodeHooks, WeakTreeNode};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::ops::Deref;
use style_types::{Property, Value};

pub struct NodePtr(pub TreeNode<Node>);

pub struct Node {
    owner_document: RefCell<Option<WeakTreeNode<Node>>>,
    data: Option<NodeData>,
    computed_styles: RefCell<HashMap<Property, Value>>,
}

#[enum_dispatch(NodeHooks)]
pub enum NodeData {
    Element(Element),
    Text(Text),
    Document(Document),
    Comment(Comment),
}

pub struct InsertContext {
    pub document: NodePtr,
    pub current_node: NodePtr,
    pub parent_node: NodePtr,
}

pub struct ChildrenUpdateContext {
    pub document: NodePtr,
    pub current_node: NodePtr,
}

#[enum_dispatch]
pub trait NodeHooks {
    #[allow(unused_variables)]
    fn on_inserted(&self, context: InsertContext) {}
    #[allow(unused_variables)]
    fn on_children_updated(&self, context: ChildrenUpdateContext) {}
}

impl core::fmt::Debug for Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let inner = match &self.data {
            Some(data) => format!("{:?}", data),
            None => "[Empty Node]".to_string(),
        };
        write!(f, "{}", inner)
    }
}

impl core::fmt::Debug for NodePtr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Clone for NodePtr {
    fn clone(&self) -> Self {
        NodePtr(self.0.clone())
    }
}

impl TreeNodeHooks<Node> for Node {
    fn on_inserted(&self, current: TreeNode<Node>, parent: TreeNode<Node>) {
        if let Some(data) = &self.data {
            if let Some(document) = self.owner_document() {
                let context = InsertContext {
                    document: NodePtr(document),
                    current_node: NodePtr(current),
                    parent_node: NodePtr(parent),
                };
                data.handle_on_inserted(context);
            }
        }
    }

    fn on_children_updated(&self, current: TreeNode<Node>) {
        if let Some(data) = &self.data {
            if let Some(document) = self.owner_document() {
                let context = ChildrenUpdateContext {
                    document: NodePtr(document),
                    current_node: NodePtr(current),
                };
                data.handle_on_children_updated(context);
            }
        }
    }
}

impl NodeData {
    pub fn handle_on_inserted(&self, context: InsertContext) {
        self.on_inserted(context);
    }

    pub fn handle_on_children_updated(&self, context: ChildrenUpdateContext) {
        self.on_children_updated(context);
    }
}

impl core::fmt::Debug for NodeData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NodeData::Text(text) => write!(f, "Text({:?})", text.get_data()),
            NodeData::Comment(comment) => write!(f, "Comment({:?})", comment.get_data()),
            NodeData::Document(_) => write!(f, "Document"),
            NodeData::Element(element) => write!(f, "{:?}", element),
        }
    }
}

impl Deref for NodePtr {
    type Target = TreeNode<Node>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NodePtr {
    /// Descendant text content of the node
    /// https://dom.spec.whatwg.org/#concept-descendant-text-content
    pub fn descendant_text_content(&self) -> String {
        if let Some(text) = self.as_text_opt() {
            return text.get_data();
        }
        let mut result = String::new();
        self.for_each_child(|child| {
            result.push_str(&NodePtr(child).descendant_text_content());
        });
        result
    }

    /// Child text content of the node
    /// https://dom.spec.whatwg.org/#concept-child-text-content
    pub fn child_text_content(&self) -> String {
        let mut result = String::new();
        self.for_each_child(|child| {
            if let Some(text) = child.as_text_opt() {
                result.push_str(&text.get_data());
            }
        });
        result
    }

    pub fn child_nodes(&self) -> NodeList {
        NodeList::new(self.first_child())
    }

    pub fn to_string(&self) -> String {
        self.to_string_with_indent(0)
    }

    fn to_string_with_indent(&self, indent_level: usize) -> String {
        let child_nodes = self.child_nodes();
        let mut result = format!(
            "{}{:#?}({} child)\n",
            "    ".repeat(indent_level),
            self,
            child_nodes.length()
        );
        for node in child_nodes {
            let child_result = NodePtr(node).to_string_with_indent(indent_level + 1);
            result.push_str(&child_result);
        }
        result
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
            owner_document: RefCell::new(None),
            data: None,
            computed_styles: RefCell::new(HashMap::new()),
        }
    }

    /// Set the owner document for node
    pub fn set_document(&self, doc: WeakTreeNode<Node>) {
        self.owner_document.replace(Some(doc));
    }

    /// Owner document of the node
    pub fn owner_document(&self) -> Option<TreeNode<Node>> {
        match self.owner_document.borrow().deref() {
            Some(node) => node.upgrade(),
            _ => None,
        }
    }

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

    pub fn set_computed_styles(&self, computed_styles: HashMap<Property, Value>) {
        *self.computed_styles.borrow_mut() = computed_styles;
    }

    pub fn computed_styles(&self) -> Ref<HashMap<Property, Value>> {
        self.computed_styles.borrow()
    }

    pub fn get_style(&self, property: &Property) -> Value {
        self.computed_styles()
            .get(property)
            .expect(&format!("Unavailable style for :{:?}", property))
            .clone()
    }
}
