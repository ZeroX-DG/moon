use super::ElementHooks;
use super::ElementMethods;
use crate::node::ChildrenUpdateContext;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLTitleElement {}

impl HTMLTitleElement {
    pub fn empty() -> Self {
        Self {}
    }
}

impl ElementHooks for HTMLTitleElement {}

impl NodeHooks for HTMLTitleElement {
    fn on_children_updated(&self, context: ChildrenUpdateContext) {
        let title = context.current_node.descendant_text_content();
        context.document.as_document().set_title(title);
    }
}

impl ElementMethods for HTMLTitleElement {
    fn tag_name(&self) -> String {
        "title".to_string()
    }
}
