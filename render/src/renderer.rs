use super::page::Page;
use gfx::{Bitmap, Canvas};
use painting::Painter;
use shared::primitive::Size;
use url::Url;

pub struct Renderer<'a> {
    painter: Painter<Canvas<'a>>,
    page: Page,
}

pub struct RendererInitializeParams {
    pub viewport: Size,
}

impl<'a> Renderer<'a> {
    pub async fn new() -> Renderer<'a> {
        Self {
            painter: Painter::new(Canvas::new().await),
            page: Page::new(),
        }
    }

    pub fn initialize(&mut self, params: RendererInitializeParams) {
        self.page.resize(params.viewport.clone());
        self.painter.resize(params.viewport.clone());
    }

    pub fn load_html(&mut self, html: String, base_url: Url) {
        self.page.load_html(html, base_url);
    }

    pub fn paint(&mut self) {
        let main_frame = self.page.main_frame();

        if let Some(layout_root) = main_frame.layout().layout_tree() {
            self.painter.paint(layout_root);
        }
    }

    pub async fn output(&mut self) -> Bitmap {
        self.painter.output().await
    }
}
