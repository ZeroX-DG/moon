use super::node::NodeHooks;
use css::cssom::stylesheet::StyleSheet;
use document_loader::DocumentLoader;
use url::Url;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub struct Document {
    doctype: RefCell<Option<DocumentType>>,
    mode: RefCell<QuirksMode>,
    loader: RefCell<Option<Rc<RefCell<dyn DocumentLoader>>>>,
    stylesheets: RefCell<Vec<Rc<StyleSheet>>>,
    base: RefCell<Option<Url>>
}

pub struct DocumentType {
    name: String,
    public_id: String,
    system_id: String,
}

#[derive(Clone)]
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
            doctype: RefCell::new(None),
            mode: RefCell::new(QuirksMode::NoQuirks),
            loader: RefCell::new(None),
            stylesheets: RefCell::new(Vec::new()),
            base: RefCell::new(None)
        }
    }

    pub fn set_doctype(&self, doctype: DocumentType) {
        *self.doctype.borrow_mut() = Some(doctype);
    }

    pub fn set_mode(&self, mode: QuirksMode) {
        *self.mode.borrow_mut() = mode;
    }

    pub fn get_mode(&self) -> QuirksMode {
        self.mode.borrow().clone()
    }

    pub fn loader(&self) -> Option<Rc<RefCell<dyn DocumentLoader>>> {
        self.loader.borrow().clone()
    }

    pub fn set_loader<L: DocumentLoader + 'static>(&self, loader: L) {
        *self.loader.borrow_mut() = Some(Rc::new(RefCell::new(loader)));
    }

    pub fn append_stylesheet(&self, stylesheet: StyleSheet) {
        self.stylesheets.borrow_mut().push(Rc::new(stylesheet));
    }

    pub fn stylesheets(&self) -> Vec<Rc<StyleSheet>> {
        self.stylesheets.borrow().deref().to_vec()
    }

    pub fn base(&self) -> Option<Url> {
        self.base.borrow().deref().clone()
    }

    pub fn set_base(&self, base: Option<Url>) {
        *self.base.borrow_mut() = base;
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
