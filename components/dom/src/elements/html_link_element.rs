use super::ElementHooks;
use super::ElementMethods;
use crate::document_loader::LoadRequest;
use crate::dom_ref::NodeRef;
use crate::node::NodeHooks;
use url::Url;

#[derive(Debug)]
pub struct HTMLLinkElement {
    href: Option<Url>,
}

impl HTMLLinkElement {
    pub fn empty() -> Self {
        Self { href: None }
    }
}

impl ElementHooks for HTMLLinkElement {
    fn on_attribute_change(&mut self, attr: &str, value: &str) {
        if attr == "href" {
            self.href = Url::parse(value).ok();
        }
    }
}

impl NodeHooks for HTMLLinkElement {
    fn on_inserted(&mut self, document: NodeRef) {
        if let Some(url) = &self.href {
            let request = LoadRequest::new(url, |bytes| String::from_utf8(bytes).unwrap())
                .on_success(|css| {
                    let tokenizer = css::tokenizer::Tokenizer::new(css.chars());
                    let mut parser =
                        css::parser::Parser::<css::tokenizer::token::Token>::new(tokenizer.run());
                    let stylesheet = parser.parse_a_css_stylesheet();

                    document
                        .borrow_mut()
                        .as_document_mut()
                        .append_stylesheet(stylesheet);
                })
                .on_error(|e| println!("Unable to load CSS: {} ({})", e, url.raw()));

            let loader = document.borrow().as_document().loader();
            loader.borrow_mut().load(request);
        }
    }
}

impl ElementMethods for HTMLLinkElement {
    fn tag_name(&self) -> String {
        "link".to_string()
    }
}
