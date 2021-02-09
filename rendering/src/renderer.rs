use super::layout_engine::LayoutEngine;
use super::paint::Painter;
use super::parsing::{parse_css, parse_html};
use dom::dom_ref::NodeRef;
use layout::box_model::Rect;
use message::MessageToRenderer;
use std::io::Read;

pub struct Renderer {
    id: u16,
    document: Option<NodeRef>,
    layout_engine: LayoutEngine,
    painter: Painter,
    viewport: Rect,
}

impl Renderer {
    pub async fn new(id: u16, viewport: Rect) -> Self {
        let painter = Painter::new(viewport.width as u32, viewport.height as u32).await;
        Self {
            id,
            document: None,
            layout_engine: LayoutEngine::new(viewport.clone()),
            painter,
            viewport,
        }
    }

    pub fn handle_msg(&mut self, msg: MessageToRenderer) {
        match msg {
            MessageToRenderer::LoadHTMLLocal(path) => {
                let mut html_file = std::fs::File::open(path).expect("Unable to open HTML file");
                self.load_html(&mut html_file);
            }
            MessageToRenderer::LoadCSSLocal(path) => {
                let mut css_file = std::fs::File::open(path).expect("Unable to open HTML file");
                self.load_css(&mut css_file);
            }
            _ => {}
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn load_html(&mut self, input: &mut dyn Read) {
        let mut html = String::new();
        input.read_to_string(&mut html).expect("Cannot read HTML");

        let dom = parse_html(html);

        self.document = Some(dom.clone());

        self.layout_engine.load_dom_tree(&dom);
    }

    pub fn load_css(&mut self, input: &mut dyn Read) {
        let mut css = String::new();
        input.read_to_string(&mut css).expect("Cannot read CSS");

        let style = parse_css(css);

        self.layout_engine.append_stylesheet(style);
    }

    pub async fn repaint(&mut self) -> Option<Vec<u8>> {
        if let Some(layout_tree) = self.layout_engine.layout_tree() {
            let display_list = painting::build_display_list(layout_tree);
            painting::paint(&display_list, &mut self.painter);

            self.painter.paint().await
        } else {
            None
        }
    }

    pub fn viewport(&self) -> &Rect {
        &self.viewport
    }
}
