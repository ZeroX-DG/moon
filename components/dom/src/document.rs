use super::node::{NodeHooks, NodePtr};
use css::cssom::css_rule::CSSRule;
use loader::document_loader::DocumentLoader;
use std::cell::RefCell;
use std::ops::Deref;
use style_types::{ContextualRule, ContextualStyleSheet};
use url::Url;

pub struct Document {
    title: RefCell<String>,
    doctype: RefCell<Option<DocumentType>>,
    mode: RefCell<QuirksMode>,
    loader: RefCell<Option<DocumentLoader>>,
    base: RefCell<Option<Url>>,
    style_elements: RefCell<Vec<NodePtr>>,
    user_agent_stylesheet: RefCell<Option<ContextualStyleSheet>>,
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
            title: RefCell::new(String::new()),
            doctype: RefCell::new(None),
            mode: RefCell::new(QuirksMode::NoQuirks),
            loader: RefCell::new(None),
            base: RefCell::new(None),
            style_elements: RefCell::new(Vec::new()),
            user_agent_stylesheet: RefCell::new(None),
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

    pub fn set_title(&self, title: String) {
        *self.title.borrow_mut() = title.clone();
    }

    pub fn title(&self) -> String {
        self.title.borrow().deref().clone()
    }

    pub fn loader(&self) -> DocumentLoader {
        self.loader.borrow().as_ref().unwrap().clone()
    }

    pub fn set_loader(&self, loader: DocumentLoader) {
        self.loader.borrow_mut().replace(loader);
    }

    pub fn set_user_agent_stylesheet(&self, stylesheet: ContextualStyleSheet) {
        self.user_agent_stylesheet.borrow_mut().replace(stylesheet);
    }

    pub fn register_style_element(&self, element: NodePtr) {
        self.style_elements.borrow_mut().push(element);
    }

    pub fn style_rules(&self) -> Vec<ContextualRule> {
        let style_elements = self.style_elements.borrow();
        let mut style_rules = Vec::new();

        fn stylesheet_to_rules(stylesheet: &ContextualStyleSheet) -> Vec<ContextualRule> {
            stylesheet
                .inner
                .iter()
                .map(move |rule| match rule {
                    CSSRule::Style(style) => ContextualRule {
                        inner: style.clone(),
                        location: stylesheet.location.clone(),
                        origin: stylesheet.origin.clone(),
                    },
                })
                .collect()
        }

        if let Some(stylesheet) = &*self.user_agent_stylesheet.borrow() {
            style_rules.extend(stylesheet_to_rules(stylesheet));
        }

        for element in style_elements.iter() {
            let element = element.as_element();

            let rules = match element.data() {
                crate::elements::ElementData::Link(link) => {
                    link.stylesheet().lock().unwrap().as_ref().map(stylesheet_to_rules).unwrap_or(Vec::new())
                }
                crate::elements::ElementData::Style(style) => {
                    style.stylesheet().as_ref().map(stylesheet_to_rules).unwrap_or(Vec::new())
                }
                _ => Vec::new()
            };

            style_rules.extend(rules);
        }
        style_rules
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
