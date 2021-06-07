use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;
use url::Url;

#[derive(Debug)]
pub struct HTMLAnchorElement {
    href: Option<Url>,
}

impl HTMLAnchorElement {
    pub fn empty() -> Self {
        Self { href: None }
    }
}

impl ElementHooks for HTMLAnchorElement {
    fn on_attribute_change(&mut self, attr: &str, value: &str) {
        if attr == "href" {
            self.href = Url::parse(value).ok();
        }
    }
}

impl NodeHooks for HTMLAnchorElement {}

impl ElementMethods for HTMLAnchorElement {
    fn tag_name(&self) -> String {
        "a".to_string()
    }
}
