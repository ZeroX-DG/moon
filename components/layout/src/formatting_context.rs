use std::{cell::RefCell, fmt::Debug, rc::Rc};

use shared::{primitive::*, tree_node::WeakTreeNode};
use style_types::{
    Property,
    Value,
    values::{display::InnerDisplayType, prelude::Display},
};

use crate::{
    flow::{block::BlockFormattingContext, inline::InlineFormattingContext},
    layout_box::{LayoutBox, LayoutBoxPtr},
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
    pub establish_by: RefCell<WeakTreeNode<LayoutBox>>,
}

pub trait FormattingContext: Debug {
    fn base(&self) -> &BaseFormattingContext;
    fn run(&self, context: &LayoutContext, node: LayoutBoxPtr);
    fn layout_inside(
        &self,
        context: &LayoutContext,
        node: LayoutBoxPtr,
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
    establish_by: LayoutBoxPtr,
) -> Rc<dyn FormattingContext> {
    let base_context = BaseFormattingContext {
        context_type: context_type.clone(),
        establish_by: RefCell::new(WeakTreeNode::from(&establish_by.0)),
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

fn use_context(context: Rc<dyn FormattingContext>, node: LayoutBoxPtr) {
    node.formatting_context.replace(Some(context));
}

fn get_formatting_context_type(layout_node: LayoutBoxPtr) -> FormattingContextType {
    if layout_node.is_anonymous() {
        if layout_node.children_are_inline() {
            return FormattingContextType::InlineFormattingContext;
        }
        return FormattingContextType::BlockFormattingContext;
    }

    let node = layout_node.node().unwrap();

    let display = node.get_style(&Property::Display);
    let inner_display = match display {
        Value::Display(Display::Full(_, ref inner)) => inner,
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
    node: LayoutBoxPtr,
) -> Option<Rc<dyn FormattingContext>> {
    if !node.can_have_children() {
        return None;
    }

    let formatting_context_type = get_formatting_context_type(node.clone());

    if let FormattingContextType::BlockFormattingContext = formatting_context_type {
        let base_context = BaseFormattingContext {
            context_type: formatting_context_type,
            establish_by: RefCell::new(WeakTreeNode::from(&node.0)),
        };
        return Some(Rc::new(BlockFormattingContext::new(base_context)));
    }

    if node.children_are_inline() {
        let base_context = BaseFormattingContext {
            context_type: FormattingContextType::InlineFormattingContext,
            establish_by: RefCell::new(WeakTreeNode::from(&node.0)),
        };
        return Some(Rc::new(InlineFormattingContext::new(base_context)));
    }

    return None;
}
