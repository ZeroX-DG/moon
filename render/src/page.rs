use dom::{
    document::Document,
    node::{Node, NodeData, NodePtr},
};
use shared::{primitive::Size, tree_node::TreeNode};
use style_types::{CSSLocation, CascadeOrigin, ContextualStyleSheet};
use url::Url;

use super::frame::Frame;

const USER_AGENT_STYLES: &str = include_str!("./html.css");

pub struct Page {
    main_frame: Frame,
}

impl Page {
    pub fn new() -> Self {
        Self {
            main_frame: Frame::new(),
        }
    }

    pub fn main_frame(&self) -> &Frame {
        &self.main_frame
    }

    pub fn resize(&mut self, size: Size) {
        self.main_frame.resize(size);
    }

    pub fn load_html(&mut self, html: String, base_url: Url) {
        let document = NodePtr(TreeNode::new(Node::new(
            NodeData::Document(Document::new()),
        )));

        let tokenizer = css::tokenizer::Tokenizer::new(USER_AGENT_STYLES.chars());
        let mut parser = css::parser::Parser::<css::tokenizer::token::Token>::new(tokenizer.run());
        let stylesheet = parser.parse_a_css_stylesheet();
        let stylesheet =
            ContextualStyleSheet::new(stylesheet, CascadeOrigin::UserAgent, CSSLocation::External);
        document.as_document().append_stylesheet(stylesheet);

        log::debug!("Base URL: {}", base_url);
        document.as_document().set_base(Some(base_url));

        let tokenizer = html::tokenizer::Tokenizer::new(html.chars());
        let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer, document);
        let document = tree_builder.run();

        self.main_frame.set_document(document);
    }
}
