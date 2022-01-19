use std::{cell::RefCell, rc::{Rc, Weak}};

use shared::primitive::*;

use crate::layout_box::LayoutBox;

pub struct LayoutContext {
    pub viewport: Rect,
}

#[derive(Debug)]
pub enum FormattingContextType {
    BlockFormattingContext,
    InlineFormattingContext
}

#[derive(Debug)]
pub struct FormattingContext {
    pub context_type: FormattingContextType,
    pub establish_by: RefCell<Option<Weak<LayoutBox>>>
}

impl FormattingContext {
    pub fn run(&self, context: Rc<LayoutContext>, boxes: &[Rc<LayoutBox>]) {
        
    }
}

pub fn establish_context(context_type: FormattingContextType, establish_by: Rc<LayoutBox>) -> Rc<FormattingContext> {
    let context = Rc::new(FormattingContext {
        context_type,
        establish_by: RefCell::new(Some(Rc::downgrade(&establish_by)))
    });
    establish_by.base.formatting_context.replace(Some(context.clone()));
    context
}

