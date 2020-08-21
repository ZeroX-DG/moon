use crate::node::NodeRef;

pub struct Document {
    doctype: Option<NodeRef>,
    mode: QuirksMode
}

pub enum QuirksMode {
    Quirks,
    NoQuirks,
    LimitedQuirks
}

impl Document {
    pub fn new() -> Self {
        Self {
            doctype: None,
            mode: QuirksMode::NoQuirks
        }
    }

    pub fn set_doctype(&mut self, doctype: Option<NodeRef>) {
        self.doctype = doctype;
    }

    pub fn set_mode(&mut self, mode: QuirksMode) {
        self.mode = mode;
    }
}

