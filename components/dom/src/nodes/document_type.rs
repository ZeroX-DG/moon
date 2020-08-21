use crate::node::NodeData;

pub struct DocumentType {
    pub name: String,
    pub publicId: String,
    pub systemId: String
}

impl NodeData for DocumentType {}
