use std::{
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
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
    fn layout_inside(
        &self,
        context: &LayoutContext,
        node: Rc<LayoutBox>,
    ) -> Option<Rc<dyn FormattingContext>> {
        if !node.can_have_children() {
            return None;
        }

        let independent_formatting_context =
            create_independent_formatting_context_if_needed(node.clone());

        if let Some(formatting_context) = &independent_formatting_context {
            formatting_context.run(context, node.clone());
        } else {
            self.run(context, node.clone());
        }

        independent_formatting_context
    }
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
        FormattingContextType::BlockFormattingContext => {
            Rc::new(BlockFormattingContext::new(base_context))
        }
        FormattingContextType::InlineFormattingContext => {
            Rc::new(InlineFormattingContext::new(base_context))
        }
    };
    use_context(context.clone(), establish_by);
    context
}

fn use_context(context: Rc<dyn FormattingContext>, node: Rc<LayoutBox>) {
    node.base.formatting_context.replace(Some(context));
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

pub fn create_independent_formatting_context_if_needed(
    node: Rc<LayoutBox>,
) -> Option<Rc<dyn FormattingContext>> {
    if !node.can_have_children() {
        return None;
    }

    let formatting_context_type = get_formatting_context_type(node.clone());

    if let FormattingContextType::BlockFormattingContext = formatting_context_type {
        let base_context = BaseFormattingContext {
            context_type: formatting_context_type,
            establish_by: RefCell::new(Some(Rc::downgrade(&node))),
        };
        return Some(Rc::new(BlockFormattingContext::new(base_context)));
    }

    if node.children_are_inline() {
        let base_context = BaseFormattingContext {
            context_type: FormattingContextType::InlineFormattingContext,
            establish_by: RefCell::new(Some(Rc::downgrade(&node))),
        };
        return Some(Rc::new(InlineFormattingContext::new(base_context)));
    }

    return None;
}
