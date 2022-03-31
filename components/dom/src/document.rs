use super::node::NodeHooks;
use css::cssom::css_rule::CSSRule;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use style_types::{ContextualRule, ContextualStyleSheet};
use url::Url;

pub struct Document {
    doctype: RefCell<Option<DocumentType>>,
    mode: RefCell<QuirksMode>,
    stylesheets: RefCell<Vec<Rc<ContextualStyleSheet>>>,
    cached_style_rules: RefCell<Vec<(Weak<ContextualStyleSheet>, Vec<ContextualRule>)>>,
    base: RefCell<Option<Url>>,
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
            stylesheets: RefCell::new(Vec::new()),
            cached_style_rules: RefCell::new(Vec::new()),
            base: RefCell::new(None),
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

    pub fn append_stylesheet(&self, stylesheet: ContextualStyleSheet) -> Rc<ContextualStyleSheet> {
        let stylesheet_ptr = Rc::new(stylesheet);
        self.stylesheets.borrow_mut().push(stylesheet_ptr.clone());
        stylesheet_ptr
    }

    pub fn remove_stylesheet(&self, stylesheet: &Rc<ContextualStyleSheet>) {
        let maybe_index = self
            .stylesheets
            .borrow()
            .iter()
            .rposition(|sheet| Rc::ptr_eq(sheet, stylesheet));

        if let Some(index) = maybe_index {
            self.stylesheets.borrow_mut().remove(index);
        }
    }

    pub fn style_rules(&self) -> Vec<ContextualRule> {
        self.gabarge_collect_values();

        let mut append_rules = Vec::new();

        for stylesheet in self.stylesheets.borrow().iter() {
            let mut is_cached = false;
            for (cached_sheet, _) in self.cached_style_rules.borrow().iter() {
                if cached_sheet.as_ptr() == Rc::as_ptr(stylesheet) {
                    is_cached = true;
                    break;
                }
            }

            if is_cached {
                continue;
            }

            let rules = stylesheet
                .inner
                .iter()
                .map(move |rule| match rule {
                    CSSRule::Style(style) => ContextualRule {
                        inner: style.clone(),
                        location: stylesheet.location.clone(),
                        origin: stylesheet.origin.clone(),
                    },
                })
                .collect();

            append_rules.push((Rc::downgrade(stylesheet), rules));
        }

        {
            let mut cached_style_rules = self.cached_style_rules.borrow_mut();
            for rules in append_rules {
                cached_style_rules.push(rules);
            }
        }

        self.cached_style_rules
            .borrow()
            .iter()
            .flat_map(|(_, rules)| rules)
            .cloned()
            .collect()
    }

    fn gabarge_collect_values(&self) {
        let mut indexes_to_remove = Vec::new();
        for (index, (stylesheet, _)) in self.cached_style_rules.borrow().iter().enumerate() {
            if stylesheet.upgrade().is_none() {
                indexes_to_remove.push(index);
            }
        }

        let mut cached_style_rules = self.cached_style_rules.borrow_mut();
        for index in indexes_to_remove {
            cached_style_rules.remove(index);
        }
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
