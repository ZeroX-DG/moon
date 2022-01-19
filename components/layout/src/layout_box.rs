use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    rc::{Rc, Weak},
};

use style::{
    property::Property,
    render_tree::RenderNode,
    value::Value,
    values::{
        display::Display,
        display::{InnerDisplayType, OuterDisplayType},
        prelude::Position,
    },
};

use crate::{
    box_model::Dimensions, flow::inline::InlineBox, formatting_context::FormattingContext,
};

#[derive(Debug)]
pub struct BaseBox {
    pub dimensions: RefCell<Dimensions>,
    pub children: RefCell<Vec<Rc<LayoutBox>>>,
    pub containing_block: RefCell<Option<Weak<LayoutBox>>>,
    pub formatting_context: RefCell<Option<Rc<FormattingContext>>>,
    pub parent: RefCell<Option<Weak<LayoutBox>>>,
}

impl BaseBox {
    pub fn new(context: Option<Rc<FormattingContext>>) -> Self {
        Self {
            dimensions: Default::default(),
            children: RefCell::new(Vec::new()),
            containing_block: RefCell::new(None),
            formatting_context: RefCell::new(context),
            parent: RefCell::new(None),
        }
    }
}

#[derive(Debug)]
pub struct LayoutBox {
    pub base: BaseBox,
    pub data: BoxData,
    pub node: Option<Rc<RenderNode>>,
}

#[derive(Debug)]
pub enum BoxData {
    BlockBox,
    InlineContents(InlineContents),
}

#[derive(Debug)]
pub enum InlineContents {
    InlineBox(InlineBox),
    TextRun,
}

impl LayoutBox {
    pub fn new(render_node: Rc<RenderNode>) -> Self {
        let box_data = {
            if render_node.node.is_text() {
                BoxData::InlineContents(InlineContents::TextRun)
            } else {
                match render_node.get_style(&Property::Display).inner() {
                    Value::Display(d) => match d {
                        Display::Full(outer, inner) => match (outer, inner) {
                            (OuterDisplayType::Block, InnerDisplayType::Flow) => BoxData::BlockBox,
                            (OuterDisplayType::Inline, InnerDisplayType::Flow)
                            | (OuterDisplayType::Inline, InnerDisplayType::FlowRoot) => {
                                BoxData::InlineContents(InlineContents::InlineBox(InlineBox::new()))
                            }
                            _ => unimplemented!("Unsupport display type: {:#?}", d),
                        },
                        _ => unimplemented!("Unsupport display type: {:#?}", d),
                    },
                    _ => unreachable!(),
                }
            }
        };

        Self {
            base: BaseBox::new(None),
            data: box_data,
            node: Some(render_node),
        }
    }

    pub fn new_anonymous(data: BoxData) -> Self {
        Self {
            base: BaseBox::new(None),
            data,
            node: None,
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.node.is_none()
    }

    pub fn parent(&self) -> Option<Rc<LayoutBox>> {
        match self.base.parent.borrow().clone() {
            Some(parent) => parent.upgrade(),
            _ => None,
        }
    }

    pub fn children(&self) -> Ref<Vec<Rc<LayoutBox>>> {
        self.base.children.borrow()
    }

    pub fn children_mut(&self) -> RefMut<Vec<Rc<LayoutBox>>> {
        self.base.children.borrow_mut()
    }

    pub fn set_children(parent: Rc<LayoutBox>, children: Vec<Rc<LayoutBox>>) {
        children
            .iter()
            .for_each(|child| child.set_parent(parent.clone()));
        parent.base.children.replace(children);
    }

    pub fn children_are_inline(&self) -> bool {
        self.base
            .children
            .borrow()
            .iter()
            .all(|child| child.is_inline())
    }

    pub fn set_parent(&self, parent: Rc<LayoutBox>) {
        self.base.parent.replace(Some(Rc::downgrade(&parent)));
        // TODO: re-calculate containing block instead of doing this.
        self.set_containing_block(parent);
    }

    pub fn set_containing_block(&self, containing_block: Rc<LayoutBox>) {
        self.base
            .containing_block
            .replace(Some(Rc::downgrade(&containing_block)));
    }

    pub fn containing_block(&self) -> Rc<LayoutBox> {
        match self.base.containing_block.borrow().clone() {
            Some(containing_block) => containing_block
                .upgrade()
                .expect("Unable to obtain containing block"),
            _ => panic!("No containing block has been set. This should not happen!"),
        }
    }

    pub fn is_inline(&self) -> bool {
        match self.data {
            BoxData::InlineContents(_) => true,
            _ => false,
        }
    }

    pub fn is_block(&self) -> bool {
        match self.data {
            BoxData::BlockBox => true,
            _ => false,
        }
    }

    pub fn is_inline_block(&self) -> bool {
        match self.render_node() {
            Some(node) => match node.get_style(&Property::Display).inner() {
                Value::Display(Display::Full(_, InnerDisplayType::FlowRoot)) => self.is_inline(),
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_positioned(&self, position: Position) -> bool {
        match self.render_node() {
            Some(node) => match node.get_style(&Property::Position).inner() {
                Value::Position(pos) => *pos == position,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_non_replaced(&self) -> bool {
        match &self.render_node() {
            Some(node) => match node.node.as_element_opt() {
                Some(e) => match e.tag_name().as_str() {
                    "video" | "image" | "img" | "canvas" => false,
                    _ => true,
                },
                _ => true,
            },
            _ => true,
        }
    }

    pub fn dimensions(&self) -> Ref<Dimensions> {
        self.base.dimensions.borrow()
    }

    pub fn dimensions_mut(&self) -> RefMut<Dimensions> {
        self.base.dimensions.borrow_mut()
    }

    pub fn render_node(&self) -> Option<Rc<RenderNode>> {
        self.node.clone()
    }

    pub fn friendly_name(&self) -> &str {
        match self.data {
            BoxData::BlockBox => "BlockBox",
            BoxData::InlineContents(InlineContents::TextRun) => "TextRun",
            BoxData::InlineContents(_) => "InlineBox",
        }
    }

    pub fn formatting_context(&self) -> Rc<FormattingContext> {
        self.base
            .formatting_context
            .borrow()
            .clone()
            .expect("No layout context! This should not happen!")
    }

    pub fn apply_explicit_sizes(&self) {
        let containing_block = self.containing_block().dimensions().content_box();

        if self.is_inline() && !self.is_inline_block() {
            return;
        }

        if let Some(render_node) = self.render_node() {
            let computed_width = render_node.get_style(&Property::Width);
            let computed_height = render_node.get_style(&Property::Height);

            if !computed_width.is_auto() {
                let used_width = computed_width.to_px(containing_block.width);
                self.base.dimensions.borrow_mut().set_width(used_width);
            }

            if !computed_height.is_auto() {
                let used_height = computed_height.to_px(containing_block.height);
                self.base.dimensions.borrow_mut().set_height(used_height);
            }
        }
    }

    pub fn add_child(parent: Rc<LayoutBox>, child: Rc<LayoutBox>) {
        parent.children_mut().push(child.clone());
        child.set_parent(parent);
    }
}
