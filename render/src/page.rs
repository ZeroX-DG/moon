use dom::{
    document::Document,
    node::{Node, NodeData, NodePtr},
};
use flume::{bounded, Sender};
use gfx::Bitmap;
use loader::{
    document_loader::DocumentLoader,
    resource_loop::{
        error::LoadError,
        request::{FetchListener, LoadRequest},
    },
};
use shared::{byte_string::ByteString, primitive::Size, tree_node::TreeNode};
use style_types::{CSSLocation, CascadeOrigin, ContextualStyleSheet};
use url::{parser::URLParser, Url};

use crate::pipeline::Pipeline;

use super::frame::Frame;

const USER_AGENT_STYLES: &str = include_str!("./html.css");

pub struct Page<'a> {
    url: Option<Url>,
    main_frame: Frame,
    pipeline: Pipeline<'a>,
}

impl<'a> Page<'a> {
    pub async fn new(init_size: Size) -> Page<'a> {
        Page {
            url: None,
            main_frame: Frame::new(init_size),
            pipeline: Pipeline::new().await,
        }
    }

    pub async fn resize(&mut self, size: Size) {
        self.main_frame.resize(size, &mut self.pipeline).await;
    }

    pub async fn scroll(&mut self, y: f32) {
        self.main_frame.scroll(y, &mut self.pipeline).await;
    }

    pub async fn handle_mouse_move(&mut self, coord: shared::primitive::Point) {
        self.main_frame
            .handle_mouse_move(coord, &mut self.pipeline)
            .await;
    }

    pub async fn load_html(
        &mut self,
        html: String,
        base_url: Url,
        resource_loop_tx: Sender<LoadRequest>,
    ) {
        let document = NodePtr(TreeNode::new(Node::new(
            NodeData::Document(Document::new()),
        )));

        document
            .as_document()
            .set_loader(DocumentLoader::new(resource_loop_tx));

        let tokenizer = css::tokenizer::Tokenizer::new(USER_AGENT_STYLES.chars());
        let mut parser = css::parser::Parser::<css::tokenizer::token::Token>::new(tokenizer.run());
        let stylesheet = parser.parse_a_css_stylesheet();
        let stylesheet =
            ContextualStyleSheet::new(stylesheet, CascadeOrigin::UserAgent, CSSLocation::External);
        document.as_document().set_user_agent_stylesheet(stylesheet);

        log::debug!("Base URL: {}", base_url);
        document.as_document().set_base(Some(base_url));

        let tokenizer = html::tokenizer::Tokenizer::new(html.chars());
        let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer, document);
        let document = tree_builder.run();

        self.main_frame
            .set_document(document, &mut self.pipeline)
            .await;
    }

    pub async fn load_raw_url(&mut self, url: String, resource_loop_tx: Sender<LoadRequest>) {
        match URLParser::parse(&url, None) {
            Some(url) => self.load_url(url, resource_loop_tx).await,
            None => {
                self.show_error(
                    "Invalid URL",
                    "Error while trying to navigate to an invalid URL.",
                )
                .await
            }
        }
    }

    pub async fn load_url(&mut self, url: Url, resource_loop_tx: Sender<LoadRequest>) {
        self.url = Some(url.clone());
        match self.fetch_html(DocumentLoader::new(resource_loop_tx.clone()), url.clone()) {
            Ok(html) => {
                self.load_html(html, url, resource_loop_tx).await;
            }
            Err(e) => {
                self.show_error("Oh no!", &format!("Error while loading page: {:?}", e))
                    .await;
            }
        }
    }

    pub async fn reload(&mut self, resource_loop_tx: Sender<LoadRequest>) {
        if let Some(url) = &self.url {
            self.load_url(url.clone(), resource_loop_tx).await;
        }
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

    pub fn url(&self) -> Option<Url> {
        self.url.clone()
    }

    fn fetch_html(&self, document_loader: DocumentLoader, url: Url) -> Result<String, LoadError> {
        struct HTMLLoaderContext {
            url: Url,
            html_tx: Sender<Result<String, LoadError>>,
        }

        impl FetchListener for HTMLLoaderContext {
            fn on_finished(&self, bytes: loader::resource_loop::request::Bytes) {
                if self.url.scheme == "view-source" {
                    let raw_html = ByteString::new(&bytes).to_string();
                    let raw_html_encoded = html_escape::encode_text(&raw_html);
                    self.html_tx
                        .send(Ok(format!("<pre>{}</pre>", raw_html_encoded)))
                        .unwrap();
                    return;
                }
                self.html_tx
                    .send(Ok(ByteString::new(&bytes).to_string()))
                    .unwrap();
            }

            fn on_errored(&self, error: LoadError) {
                self.html_tx.send(Err(error)).unwrap();
            }
        }

        let (tx, rx) = bounded(1);
        document_loader.fetch(url.clone(), HTMLLoaderContext { html_tx: tx, url });

        rx.recv().expect("Failed to receive HTML")
    }

    fn get_error_page_content(&self, title: &str, error: &str) -> String {
        format!(
            "
            <html>
                <style>
                    body {{ background-color: #262ded }}
                    #error-content {{
                        width: 500px;
                        margin: 0 auto;
                        margin-top: 50px;
                        color: white;
                    }}
                </style>
                <div id='error-content'>
                    <h1>{}</h1>
                    <p>{}</p>
                </div>
            </html>
        ",
            title, error
        )
    }

    async fn show_error(&mut self, title: &str, error: &str) {
        let error_page = self.get_error_page_content(title, error);
        let document = NodePtr(TreeNode::new(Node::new(
            NodeData::Document(Document::new()),
        )));

        let tokenizer = css::tokenizer::Tokenizer::new(USER_AGENT_STYLES.chars());
        let mut parser = css::parser::Parser::<css::tokenizer::token::Token>::new(tokenizer.run());
        let stylesheet = parser.parse_a_css_stylesheet();
        let stylesheet =
            ContextualStyleSheet::new(stylesheet, CascadeOrigin::UserAgent, CSSLocation::External);
        document.as_document().set_user_agent_stylesheet(stylesheet);

        let tokenizer = html::tokenizer::Tokenizer::new(error_page.chars());
        let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer, document);
        let document = tree_builder.run();

        self.main_frame
            .set_document(document, &mut self.pipeline)
            .await;
    }
}
