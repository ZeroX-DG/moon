mod command;
mod paint_functions;
mod painter;
mod render;
mod utils;

use command::{DisplayCommand, DrawCommand};
use layout::layout_box::{LayoutNodeId, LayoutTree};
use render::PaintChainBuilder;

pub use painter::Painter;
pub use render::DisplayList;

use paint_functions::*;

pub fn paint(display_list: DisplayList, painter: &mut dyn Painter) {
    for command in display_list {
        match command {
            DisplayCommand::Draw(draw_command) => draw(draw_command, painter),
            DisplayCommand::GroupDraw(draw_commands) => {
                for draw_command in draw_commands {
                    draw(draw_command, painter);
                }
            }
        }
    }
}

fn draw(draw_command: DrawCommand, painter: &mut dyn Painter) {
    match draw_command {
        DrawCommand::FillRect(rect, color) => painter.fill_rect(rect, color),
        DrawCommand::FillRRect(rect, color) => painter.fill_rrect(rect, color),
    }
}

pub fn build_display_list(root: &LayoutNodeId, layout_tree: &LayoutTree) -> DisplayList {
    let chain = PaintChainBuilder::new_chain()
        .with_function(&paint_border)
        .with_function(&paint_background)
        .build(layout_tree);

    chain.paint(root)
}
