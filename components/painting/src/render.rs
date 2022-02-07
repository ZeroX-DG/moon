use std::rc::Rc;

use crate::command::DisplayCommand;
use layout::layout_box::LayoutBox;

pub type PaintFn = dyn Fn(Rc<LayoutBox>) -> Option<DisplayCommand>;
pub type DisplayList = Vec<DisplayCommand>;

pub struct PaintChain<'a> {
    chain: Vec<&'a PaintFn>,
}

pub struct PaintChainBuilder<'a> {
    paint_fns: Vec<&'a PaintFn>,
}

impl<'a> PaintChain<'a> {
    pub fn paint(&self, layout_node: Rc<LayoutBox>) -> DisplayList {
        let mut result = Vec::new();

        for paint_fn in &self.chain {
            if let Some(command) = paint_fn(layout_node.clone()) {
                result.push(command);
            }
        }

        for child in layout_node.children().iter() {
            result.extend(self.paint(child.clone()));
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

    pub fn with_function(mut self, paint_fn: &'a PaintFn) -> Self {
        self.paint_fns.push(paint_fn);
        self
    }

    pub fn build(self) -> PaintChain<'a> {
        PaintChain {
            chain: self.paint_fns,
        }
    }
}
