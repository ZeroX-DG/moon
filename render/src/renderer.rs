use super::page::Page;
use gfx::{Bitmap, Canvas};
use painting::Painter;
use shared::primitive::Size;
use tokio::runtime::Runtime;
use url::Url;

pub struct Renderer<'a> {
    painter: Painter<Canvas<'a>>,
    pub page: Page,

    new_title_handler: Option<Box<dyn Fn(String)>>,
}

pub struct RendererInitializeParams {
    pub viewport: Size,
}

impl<'a> Renderer<'a> {
    pub fn new() -> Renderer<'a> {
        let rt = Runtime::new().unwrap();
        Self {
            painter: Painter::new(rt.block_on(Canvas::new())),
            page: Page::new(),
            new_title_handler: None,
        }
    }

    pub fn initialize(&mut self, params: RendererInitializeParams) {
        self.resize(params.viewport);
    }

    pub fn resize(&mut self, size: Size) {
        self.page.resize(size.clone());
        self.painter.resize(size.clone());
    }

    pub fn load_html(&mut self, html: String, base_url: Url) {
        self.page.load_html(html, base_url);
        if let Some(handler) = &self.new_title_handler {
            handler(self.page.main_frame().document().as_document().title());
        }
        self.paint();
    }

    pub fn output(&mut self) -> Bitmap {
        let rt = Runtime::new().unwrap();
        rt.block_on(self.painter.output())
    }

    pub fn on_new_title(&mut self, handler: impl Fn(String) + 'static) {
        self.new_title_handler = Some(Box::new(handler));
    }

    pub fn paint(&mut self) {
        let main_frame = self.page.main_frame();

        if let Some(layout_root) = main_frame.layout().layout_tree() {
            self.painter.paint(layout_root);
        }
    }
}
