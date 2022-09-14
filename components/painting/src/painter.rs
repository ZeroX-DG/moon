use gfx::Graphics;
use layout::layout_box::LayoutBoxPtr;
use shared::primitive::{Point, Rect, Size};

use crate::display_list::{Borders, Command, DisplayListBuilder};

pub struct Painter<G: Graphics> {
    gfx: G,
    canvas_size: Size,
    clip_rects: Vec<Rect>,
}

impl<G: Graphics> Painter<G> {
    pub fn new(gfx: G) -> Self {
        Self {
            gfx,
            canvas_size: Size::default(),
            clip_rects: Vec::new(),
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.gfx.resize(size.clone());
        self.canvas_size = size;
    }

    pub async fn output(&mut self) -> Vec<u8> {
        let result = self.gfx.output().await;
        result
    }

    pub fn paint(&mut self, layout_box: &LayoutBoxPtr) {
        let display_list = DisplayListBuilder::new(&self.canvas_size).build(layout_box);

        for command in display_list.commands() {
            match command {
                Command::FillRect(rect, color) => self.gfx.fill_rect(self.clip_rect(rect), color),
                Command::FillRRect(rect, color) => self.gfx.fill_rrect(rect, color),
                Command::FillBorder(rect, border_rect, borders) => {
                    self.paint_borders(rect, border_rect, borders)
                }
                Command::FillText(content, rect, color, font_size) => {
                    self.gfx
                        .fill_text(content, self.clip_rect(rect), color, font_size)
                }
                Command::ClipRect(rect) => self.clip_rects.push(rect),
                Command::EndClipRect => {
                    self.clip_rects.pop();
                }
            }
        }
    }

    fn clip_rect(&self, rect: Rect) -> Rect {
        let mut used_rect = rect;

        for clipping_rect in self.clip_rects.iter().rev() {
            if !used_rect.is_overlap_rect(clipping_rect) {
                // If the current rect is not within the clipping rect region, don't render it.
                return Rect::new(0., 0., 0., 0.);
            }
            used_rect.intersect(clipping_rect);
        }

        used_rect
    }

    fn paint_borders(&mut self, box_rect: Rect, border_rect: Rect, borders: Borders) {
        self.paint_border_edges(&box_rect, &border_rect, &borders);
        self.paint_border_corners(&box_rect, &border_rect, &borders);
    }

    fn paint_border_corners(&mut self, box_rect: &Rect, border_rect: &Rect, borders: &Borders) {
        if let (Some(border_top), Some(border_left)) = (&borders.top, &borders.left) {
            self.gfx.fill_polygon(
                vec![
                    Point::new(border_rect.x, border_rect.y),
                    Point::new(box_rect.x, border_rect.y),
                    Point::new(box_rect.x, box_rect.y),
                ],
                border_top.color.clone(),
            );

            self.gfx.fill_polygon(
                vec![
                    Point::new(border_rect.x, border_rect.y),
                    Point::new(box_rect.x, box_rect.y),
                    Point::new(border_rect.x, box_rect.y),
                ],
                border_left.color.clone(),
            );
        }

        if let (Some(border_left), Some(border_bottom)) = (&borders.left, &borders.bottom) {
            self.gfx.fill_polygon(
                vec![
                    Point::new(border_rect.x, border_rect.y + border_rect.height),
                    Point::new(box_rect.x, box_rect.y + box_rect.height),
                    Point::new(box_rect.x, border_rect.y + border_rect.height),
                ],
                border_bottom.color.clone(),
            );

            self.gfx.fill_polygon(
                vec![
                    Point::new(border_rect.x, border_rect.y + border_rect.height),
                    Point::new(box_rect.x, box_rect.y + box_rect.height),
                    Point::new(border_rect.x, box_rect.y + box_rect.height),
                ],
                border_left.color.clone(),
            );
        }

        if let (Some(border_right), Some(border_bottom)) = (&borders.right, &borders.bottom) {
            self.gfx.fill_polygon(
                vec![
                    Point::new(
                        border_rect.x + border_rect.width,
                        border_rect.y + border_rect.height,
                    ),
                    Point::new(box_rect.x + box_rect.width, box_rect.y + box_rect.height),
                    Point::new(
                        box_rect.x + box_rect.width,
                        border_rect.y + border_rect.height,
                    ),
                ],
                border_bottom.color.clone(),
            );

            self.gfx.fill_polygon(
                vec![
                    Point::new(
                        border_rect.x + border_rect.width,
                        border_rect.y + border_rect.height,
                    ),
                    Point::new(box_rect.x + box_rect.width, box_rect.y + box_rect.height),
                    Point::new(
                        border_rect.x + border_rect.width,
                        box_rect.y + box_rect.height,
                    ),
                ],
                border_right.color.clone(),
            );
        }

        if let (Some(border_right), Some(border_top)) = (&borders.right, &borders.top) {
            self.gfx.fill_polygon(
                vec![
                    Point::new(border_rect.x + border_rect.width, border_rect.y),
                    Point::new(box_rect.x + box_rect.width, box_rect.y),
                    Point::new(box_rect.x + box_rect.width, border_rect.y),
                ],
                border_top.color.clone(),
            );

            self.gfx.fill_polygon(
                vec![
                    Point::new(border_rect.x + border_rect.width, border_rect.y),
                    Point::new(box_rect.x + box_rect.width, box_rect.y),
                    Point::new(border_rect.x + border_rect.width, box_rect.y),
                ],
                border_right.color.clone(),
            );
        }
    }

    fn paint_border_edges(&mut self, box_rect: &Rect, border_rect: &Rect, borders: &Borders) {
        if let Some(border) = &borders.top {
            self.gfx.fill_rect(
                Rect::new(
                    box_rect.x,
                    border_rect.y,
                    box_rect.width,
                    box_rect.y - border_rect.y,
                ),
                border.color.clone(),
            );
        }

        if let Some(border) = &borders.right {
            self.gfx.fill_rect(
                Rect::new(
                    box_rect.x + box_rect.width,
                    box_rect.y,
                    (border_rect.x + border_rect.width) - (box_rect.x + box_rect.width),
                    box_rect.height,
                ),
                border.color.clone(),
            );
        }

        if let Some(border) = &borders.bottom {
            self.gfx.fill_rect(
                Rect::new(
                    box_rect.x,
                    box_rect.y + box_rect.height,
                    box_rect.width,
                    (border_rect.y + border_rect.height) - (box_rect.y + box_rect.height),
                ),
                border.color.clone(),
            );
        }

        if let Some(border) = &borders.left {
            self.gfx.fill_rect(
                Rect::new(
                    border_rect.x,
                    box_rect.y,
                    box_rect.x - border_rect.x,
                    box_rect.height,
                ),
                border.color.clone(),
            );
        }
    }
}
