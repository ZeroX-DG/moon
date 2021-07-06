use super::ElementHooks;
use super::ElementMethods;
use crate::document_loader::LoadRequest;
use crate::dom_ref::NodeRef;
use crate::node::NodeHooks;
use url::Url;

use css::parser::Parser;
use css::tokenizer::{token::Token, Tokenizer};

#[derive(Debug)]
pub struct HTMLLinkElement {
    href: Option<Url>,
    relationship: Option<HTMLLinkRelationship>,
}

#[derive(Debug)]
pub enum HTMLLinkRelationship {
    Stylesheet,
}

impl HTMLLinkElement {
    pub fn empty() -> Self {
        Self {
            href: None,
            relationship: None,
        }
    }

    pub fn load_stylesheet(&self, url: &Url, document: NodeRef) {
        let cloned_doc = document.clone();
        let raw_url = url.raw().to_string();

        log::info!("Loading stylesheet from: {}", raw_url);

        let request = LoadRequest::new(url.clone())
            .on_success(Box::new(move |bytes| {
                let css = String::from_utf8(bytes).unwrap();
                let tokenizer = Tokenizer::new(css.chars());
                let mut parser = Parser::<Token>::new(tokenizer.run());
                let stylesheet = parser.parse_a_css_stylesheet();

                cloned_doc
                    .borrow_mut()
                    .as_document_mut()
                    .append_stylesheet(stylesheet);
            }))
            .on_error(Box::new(move |e| {
                log::info!("Unable to load CSS: {} ({})", e, raw_url)
            }));

        let loader = document
            .borrow()
            .as_document()
            .loader()
            .expect("Document loader is not set");
        loader.borrow_mut().load(request);
    }
}

impl ElementHooks for HTMLLinkElement {
    fn on_attribute_change(&mut self, attr: &str, value: &str) {
        match attr {
            "href" => {
                self.href = match Url::parse(value) {
                    Ok(url) => Some(url),
                    Err(_) => {
                        log::info!("Invalid href URL: {}", value);
                        None
                    }
                }
            }
            "rel" => {
                if value == "stylesheet" {
                    self.relationship = Some(HTMLLinkRelationship::Stylesheet);
                }
            }
            _ => {}
        }
    }
}

impl NodeHooks for HTMLLinkElement {
    fn on_inserted(&mut self, document: NodeRef) {
        match &self.href {
            Some(url) => match self.relationship {
                Some(HTMLLinkRelationship::Stylesheet) => self.load_stylesheet(url, document),
                _ => {}
            },
            None => log::info!("No URL found, ignoring"),
        }
    }
}

impl ElementMethods for HTMLLinkElement {
    fn tag_name(&self) -> String {
        "link".to_string()
    }
}
