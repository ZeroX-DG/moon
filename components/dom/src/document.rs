use super::node::NodeData;
use super::str::{USVString, DOMString};

pub struct DocumentType {
    pub name: DOMString,
    pub publicId: DOMString,
    pub systemId: DOMString
}

pub struct Document {
    URL: USVString,
    doctype: Option<DocumentType>
}

impl NodeData for Document {}

impl NodeData for DocumentType {}
