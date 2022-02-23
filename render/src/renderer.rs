use super::page::Page;
use gfx::{Bitmap, Canvas};
use painting::Painter;
use shared::primitive::Size;
use tokio::runtime::Runtime;
use url::Url;

pub struct Renderer<'a> {
    painter: Painter<Canvas<'a>>,
    page: Page,
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
        }
    }

    pub fn initialize(&mut self, params: RendererInitializeParams) {
        self.page.resize(params.viewport.clone());
        self.painter.resize(params.viewport.clone());
    }

    pub fn load_html(&mut self, html: String, base_url: Url) {
        self.page.load_html(html, base_url);
        self.paint();
    }

    pub fn output(&mut self) -> Bitmap {
        let rt = Runtime::new().unwrap();
        rt.block_on(self.painter.output())
    }

    fn paint(&mut self) {
        let main_frame = self.page.main_frame();

        if let Some(layout_root) = main_frame.layout().layout_tree() {
            self.painter.paint(layout_root);
        }
    }
}
