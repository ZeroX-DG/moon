mod rect;
mod paint;
mod display_list;

use layout::layout_box::LayoutBox;
pub use rect::Rect;
pub use paint::*;
use display_list::*;

pub trait Painter {
    fn clear(&mut self);
    fn paint_rect(&mut self, rect: Rect, paint: Paint);
}

pub fn paint<P: Painter>(root: &LayoutBox, mut painter: P) {
    let display_list = build_display_list(root);

    painter.clear();

    for command in display_list {
        execute_display_command(command, &mut painter);
    }
}

