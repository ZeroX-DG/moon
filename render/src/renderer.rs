use super::frame::FrameSize;
use super::page::Page;
use super::paint::{Bitmap, Painter};

pub struct Renderer {
    painter: Painter,
    page: Page,
    output: Option<Bitmap>,
}

pub struct RendererInitializeParams {
    pub viewport: FrameSize,
}

impl Renderer {
    pub async fn new() -> Self {
        Self {
            painter: Painter::new().await,
            page: Page::new(),
            output: None,
        }
    }

    pub fn initialize(&mut self, params: RendererInitializeParams) {
        self.page.resize(params.viewport);
    }

    pub fn load_html(&mut self, html: String) {
        self.page.load_html(html);
    }

    pub fn load_css(&mut self, css: String) {
        self.page.load_css(css);
    }

    pub async fn paint(&mut self) {
        let main_frame = self.page.main_frame();
        let viewport = main_frame.size();

        if let Some(layout_root) = main_frame.layout().root() {
            let display_list = painting::build_display_list(layout_root);
            painting::paint(&display_list, &mut self.painter);

            self.output = self.painter.paint(viewport).await;
        }
    }

    pub fn output(&self) -> &Option<Bitmap> {
        &self.output
    }
}
