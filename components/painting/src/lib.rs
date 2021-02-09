mod color;
mod command;
mod paint_functions;
mod painter;
mod rect;
mod render;

use command::DisplayCommand;
use layout::layout_box::LayoutBox;
use render::PaintChainBuilder;

pub use color::*;
pub use painter::Painter;
pub use rect::*;
pub use render::DisplayList;

use paint_functions::background::paint_background;

pub fn paint(display_list: &DisplayList, painter: &mut dyn Painter) {
    for command in display_list {
        match command {
            DisplayCommand::FillRect(rect, color) => painter.fill_rect(&rect, &color),
            _ => {}
        }
    }
}

pub fn build_display_list(layout_box: &LayoutBox) -> DisplayList {
    let chain = PaintChainBuilder::new_chain()
        .then(&paint_background)
        .build();

    chain.paint(layout_box)
}
