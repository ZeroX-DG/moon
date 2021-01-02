mod values;
mod render;
mod display_command;

pub use values::*;
pub use display_command::DisplayCommand;

use layout::layout_box::LayoutBox;
use render::render_layout_box;

pub type DisplayList = Vec<DisplayCommand>;

pub trait Painter {
    fn clear(&mut self);
    fn paint_rect(&mut self, rect: &Rect, paint: &Paint);
}

pub fn paint(display_list: &DisplayList, painter: &mut dyn Painter) {
    for command in display_list {
        execute_display_command(command, painter);
    }
}

pub fn build_display_list(root: &LayoutBox) -> DisplayList {
    let mut display_list = Vec::new();

    render_layout_box(root, &mut display_list);

    display_list
}

fn execute_display_command(command: &DisplayCommand, painter: &mut dyn Painter) {
    match command {
        DisplayCommand::DrawRect(rect, paint) => painter.paint_rect(rect, paint),
    }
}
