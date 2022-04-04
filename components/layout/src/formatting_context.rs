use std::{
    cell::RefCell,
    rc::{Rc, Weak}, fmt::Debug,
};

use shared::primitive::*;
use style::{
    property::Property,
    value::Value,
    values::{display::InnerDisplayType, prelude::Display},
};

use crate::{
    flow::{block::BlockFormattingContext, inline::InlineFormattingContext},
    layout_box::LayoutBox,
};

pub struct LayoutContext {
    pub viewport: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContextType {
    BlockFormattingContext,
    InlineFormattingContext,
}

#[derive(Debug)]
pub struct BaseFormattingContext {
    pub context_type: FormattingContextType,
    pub establish_by: RefCell<Option<Weak<LayoutBox>>>,
}

pub trait FormattingContext: Debug {
    fn base(&self) -> &BaseFormattingContext;
    fn run(&self, context: &LayoutContext, node: Rc<LayoutBox>);
}


pub fn establish_context(
    context_type: FormattingContextType,
    establish_by: Rc<LayoutBox>,
) -> Rc<dyn FormattingContext> {
    let base_context = BaseFormattingContext {
        context_type: context_type.clone(),
        establish_by: RefCell::new(Some(Rc::downgrade(&establish_by))),
    };
    let context: Rc<dyn FormattingContext> = match context_type {
        FormattingContextType::BlockFormattingContext => Rc::new(BlockFormattingContext::new(base_context)),
        FormattingContextType::InlineFormattingContext => Rc::new(InlineFormattingContext::new(base_context))
    };
    use_context(context.clone(), establish_by);
    context
}

fn use_context(context: Rc<dyn FormattingContext>, node: Rc<LayoutBox>) {
    node
        .base
        .formatting_context
        .replace(Some(context));
}

fn get_formatting_context_type(layout_node: Rc<LayoutBox>) -> FormattingContextType {
    if layout_node.is_anonymous() {
        if layout_node.children_are_inline() {
            return FormattingContextType::InlineFormattingContext;
        }
        return FormattingContextType::BlockFormattingContext;
    }

    let node = layout_node.render_node().unwrap();

    let display = node.get_style(&Property::Display);
    let inner_display = match display.inner() {
        Value::Display(Display::Full(_, inner)) => inner,
        _ => unreachable!(),
    };

    match inner_display {
        InnerDisplayType::Flow => {
            if layout_node.children_are_inline() {
                FormattingContextType::InlineFormattingContext
            } else {
                FormattingContextType::BlockFormattingContext
            }
        }
        InnerDisplayType::FlowRoot => FormattingContextType::BlockFormattingContext,
        _ => unimplemented!("Unsupported display type: {:#?}", display),
    }
}

pub fn establish_context_for(node: Rc<LayoutBox>) {
    let node_context_type = get_formatting_context_type(node.clone());

    let mut reuse_context = false;

    if let Some(parent) = node.parent() {
        let parent_context = parent.formatting_context();
        if parent_context.base().context_type == node_context_type {
            use_context(parent_context, node.clone());
            reuse_context = true;
        }
    }

    if !reuse_context {
        establish_context(node_context_type, node.clone());
    }
    

    for child in node.children().iter() {
        establish_context_for(child.clone());
    }
}
