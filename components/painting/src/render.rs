use crate::command::DisplayCommand;
use layout::layout_box::LayoutBox;

pub type PaintFn = dyn Fn(&LayoutBox) -> Option<DisplayCommand>;
pub type DisplayList = Vec<DisplayCommand>;

pub struct PaintChain<'a>(Vec<&'a PaintFn>);

pub struct PaintChainBuilder<'a> {
    paint_fns: Vec<&'a PaintFn>,
}

impl<'a> PaintChain<'a> {
    pub fn paint(&self, layout_box: &LayoutBox) -> DisplayList {
        let mut result = Vec::new();

        for paint_fn in &self.0 {
            if let Some(command) = paint_fn(layout_box) {
                result.push(command);
            }
        }

        for child in &layout_box.children {
            result.extend(self.paint(child));
        }

        result
    }
}

impl<'a> PaintChainBuilder<'a> {
    pub fn new_chain() -> Self {
        Self {
            paint_fns: Vec::new(),
        }
    }

    pub fn then(mut self, paint_fn: &'a PaintFn) -> Self {
        self.paint_fns.push(paint_fn);
        self
    }

    pub fn build(self) -> PaintChain<'a> {
        PaintChain(self.paint_fns)
    }
}
