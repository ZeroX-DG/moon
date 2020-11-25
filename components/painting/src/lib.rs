mod components;
mod render;

pub use components::*;

use layout::layout_box::LayoutBox;
use render::render_layout_box;

#[derive(Debug)]
pub enum DisplayCommand {
    DrawRect(Rect, Paint),
}

pub type DisplayList = Vec<DisplayCommand>;

pub trait Painter<Canvas> {
    fn clear(&mut self, canvas: &mut Canvas);
    fn paint_rect(&mut self, rect: Rect, paint: Paint, canvas: &mut Canvas);
}

pub fn paint<P, C>(root: &LayoutBox, painter: &mut P, canvas: &mut C)
where
    P: Painter<C>,
{
    let display_list = build_display_list(root);

    painter.clear(canvas);

    for command in display_list {
        execute_display_command(command, painter, canvas);
    }
}

fn build_display_list(root: &LayoutBox) -> DisplayList {
    let mut display_list = Vec::new();

    render_layout_box(root, &mut display_list);

    display_list
}

fn execute_display_command<P, C>(command: DisplayCommand, painter: &mut P, canvas: &mut C)
where
    P: Painter<C>,
{
    match command {
        DisplayCommand::DrawRect(rect, paint) => painter.paint_rect(rect, paint, canvas),
    }
}
