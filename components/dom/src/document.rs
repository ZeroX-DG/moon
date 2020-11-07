use super::node::Node;

pub struct Document {
    pub node: Node,
    doctype: Option<DocumentType>,
    mode: QuirksMode,
}

pub struct DocumentType {
    pub node: Node,
    name: String,
    public_id: String,
    system_id: String,
}

pub enum QuirksMode {
    Quirks,
    NoQuirks,
    LimitedQuirks,
}

impl core::fmt::Debug for Document {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Document")
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            node: Node::new(),
            doctype: None,
            mode: QuirksMode::NoQuirks,
        }
    }

    pub fn set_doctype(&mut self, doctype: DocumentType) {
        self.doctype = Some(doctype);
    }

    pub fn set_mode(&mut self, mode: QuirksMode) {
        self.mode = mode;
    }

    pub fn get_mode(&self) -> &QuirksMode {
        &self.mode
    }
}

impl core::fmt::Debug for DocumentType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Doctype at {:#?}", self as *const DocumentType)
    }
}

impl DocumentType {
    pub fn new(name: String, public_id: Option<String>, system_id: Option<String>) -> Self {
        Self {
            node: Node::new(),
            name,
            public_id: public_id.unwrap_or_default(),
            system_id: system_id.unwrap_or_default(),
        }
    }
}
