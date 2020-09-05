use super::dom_ref::DOMObject;

use super::node::Node;
use super::document::{Document, DocumentType};
use super::element::Element;
use super::comment::Comment;
use super::character_data::CharacterData;

impl DOMObject for Node {
    fn as_node(&self) -> &Node {
        self
    }
}

impl DOMObject for Document {
    fn as_node(&self) -> &Node {
        &self.node
    }
}

impl DOMObject for DocumentType {
    fn as_node(&self) -> &Node {
        &self.node
    }
}

impl DOMObject for Element {
    fn as_node(&self) -> &Node {
        &self.node
    }
}

impl DOMObject for Comment {
    fn as_node(&self) -> &Node {
        &self.character_data.node
    }
}

impl DOMObject for CharacterData {
    fn as_node(&self) -> &Node {
        &self.node
    }
}

