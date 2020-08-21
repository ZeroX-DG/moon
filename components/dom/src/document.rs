use super::node::NodeData;

pub struct DocumentType {
    pub name: String,
    pub publicId: String,
    pub systemId: String
}

pub struct Document {
    URL: String,
    doctype: Option<DocumentType>
}

impl NodeData for Document {}

impl NodeData for DocumentType {}
