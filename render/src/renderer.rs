use super::frame::FrameSize;
use super::page::Page;
use gfx::{Bitmap, Painter};

pub struct Renderer<'a> {
    painter: Painter<'a>,
    page: Page,
}

pub struct RendererInitializeParams {
    pub viewport: FrameSize,
}

impl<'a> Renderer<'a> {
    pub async fn new() -> Renderer<'a> {
        Self {
            painter: Painter::new().await,
            page: Page::new(),
        }
    }

    pub fn initialize(&mut self, params: RendererInitializeParams) {
        self.page.resize(params.viewport);
        self.painter.resize(params.viewport);
    }

    pub fn load_html(&mut self, html: String) {
        self.page.load_html(html);
    }

    pub fn paint(&mut self) {
        let main_frame = self.page.main_frame();

        if let Some(layout_root) = main_frame.layout().root() {
            let display_list = painting::build_display_list(&layout_root, main_frame.layout().layout_tree());
            painting::paint(display_list, &mut self.painter);

            self.painter.paint();
        }
    }

    pub async fn output(&mut self) -> Bitmap {
        self.painter.output().await
    }
}
