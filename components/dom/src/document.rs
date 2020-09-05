use super::node::Node;

pub struct Document {
    pub node: Node,
    doctype: DocumentType,
    mode: QuirksMode
}

pub struct DocumentType {
    pub node: Node,
    name: String,
    publicId: String,
    systemId: String
}

pub enum QuirksMode {
    Quirks,
    NoQuirks,
    LimitedQuirks
}
