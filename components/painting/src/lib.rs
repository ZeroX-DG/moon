mod command;
mod paint_functions;
mod painter;
mod primitive;
mod render;
mod utils;

use command::{DisplayCommand, DrawCommand};
use layout::layout_box::LayoutBox;
use render::PaintChainBuilder;

pub use painter::Painter;
pub use primitive::*;
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

pub fn build_display_list(layout_box: &LayoutBox) -> DisplayList {
    let chain = PaintChainBuilder::new_chain()
        .with_function(&paint_border)
        .with_function(&paint_background)
        .build();

    chain.paint(layout_box)
}
