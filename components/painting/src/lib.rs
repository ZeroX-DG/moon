mod gfx_painter;
mod utils;

use std::rc::Rc;

use layout::layout_box::LayoutBox;

pub use gfx_painter::GfxPainter;
use shared::primitive::Size;
use style::property::Property;
use utils::color_from_value;

pub struct Painter<G: GfxPainter> {
    gfx: G
}

impl<G: GfxPainter> Painter<G> {
    pub fn new(gfx: G) -> Self {
        Self {
            gfx
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.gfx.resize(size);
    }

    pub async fn output(&mut self) -> Vec<u8> {
        let result = self.gfx.output().await;
        result
    }

    pub fn paint(&mut self, layout_box: Rc<LayoutBox>) {
        self.paint_background(layout_box.clone());

        for child in layout_box.children().iter() {
            self.paint(child.clone());
        }
    }

    fn paint_background(&mut self, layout_box: Rc<LayoutBox>) {
        if layout_box.is_anonymous() {
            return;
        }
        let render_node = layout_box.render_node().unwrap();
        let background_rect = layout_box.padding_box_absolute();
        let background_color = color_from_value(&render_node.get_style(&Property::BackgroundColor));

        self.gfx.fill_rect(background_rect, background_color);
    }
}

