use dom::{
    document::Document,
    node::{Node, NodeData, NodePtr},
};
use gfx::Bitmap;
use loader::ResourceLoader;
use shared::{primitive::Size, tree_node::TreeNode};
use style_types::{CSSLocation, CascadeOrigin, ContextualStyleSheet};
use url::Url;

use crate::pipeline::Pipeline;

use super::frame::Frame;

const USER_AGENT_STYLES: &str = include_str!("./html.css");

pub struct Page<'a> {
    main_frame: Frame,
    pipeline: Pipeline<'a>,
}

impl<'a> Page<'a> {
    pub async fn new(init_size: Size) -> Page<'a> {
        Page {
            main_frame: Frame::new(init_size),
            pipeline: Pipeline::new().await,
        }
    }

    pub async fn resize(&mut self, size: Size) {
        self.main_frame.resize(size, &mut self.pipeline).await;
    }

    pub async fn load_html(&mut self, html: String, base_url: Url) {
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

        self.main_frame
            .set_document(document, &mut self.pipeline)
            .await;
    }

    pub fn bitmap(&self) -> Option<&Bitmap> {
        self.main_frame.bitmap()
    }

    pub fn title(&self) -> String {
        self.main_frame
            .document()
            .map(|document| document.as_document().title())
            .unwrap_or_default()
    }
}
