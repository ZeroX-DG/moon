use super::document_loader::DocumentLoader;
use super::node::NodeHooks;
use css::cssom::stylesheet::StyleSheet;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Document {
    doctype: Option<DocumentType>,
    mode: QuirksMode,
    loader: Rc<RefCell<DocumentLoader>>,
    stylesheets: Vec<StyleSheet>,
}

pub struct DocumentType {
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

impl NodeHooks for Document {}

impl Document {
    pub fn new() -> Self {
        Self {
            doctype: None,
            mode: QuirksMode::NoQuirks,
            loader: Rc::new(RefCell::new(DocumentLoader::new())),
            stylesheets: Vec::new(),
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

    pub fn loader(&self) -> Rc<RefCell<DocumentLoader>> {
        self.loader.clone()
    }

    pub fn append_stylesheet(&mut self, stylesheet: StyleSheet) {
        self.stylesheets.push(stylesheet);
    }

    pub fn stylesheets(&self) -> &[StyleSheet] {
        &self.stylesheets
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
            name,
            public_id: public_id.unwrap_or_default(),
            system_id: system_id.unwrap_or_default(),
        }
    }
}
