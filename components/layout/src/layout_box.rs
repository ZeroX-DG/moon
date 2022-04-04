use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    rc::{Rc, Weak},
};

use shared::primitive::{Point, Rect, Size};
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

use crate::{box_model::BoxModel, flow::line_box::LineBox, formatting_context::FormattingContext};

#[derive(Debug)]
pub struct BaseBox {
    pub box_model: RefCell<BoxModel>,
    pub offset: RefCell<Point>,
    pub content_size: RefCell<Size>,
    pub children: RefCell<Vec<Rc<LayoutBox>>>,
    pub containing_block: RefCell<Option<Weak<LayoutBox>>>,
    pub formatting_context: RefCell<Option<Rc<dyn FormattingContext>>>,
    pub parent: RefCell<Option<Weak<LayoutBox>>>,
}

impl BaseBox {
    pub fn new(context: Option<Rc<dyn FormattingContext>>) -> Self {
        Self {
            box_model: Default::default(),
            offset: Default::default(),
            content_size: Default::default(),
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
    BlockBox {
        lines: RefCell<Vec<LineBox>>, // Only if the block box establish IFC
    },
    InlineContents(InlineContents),
}

#[derive(Debug)]
pub enum InlineContents {
    InlineBox,
    TextRun,
}

impl BoxData {
    pub fn block_box() -> Self {
        Self::BlockBox {
            lines: RefCell::new(Vec::new()),
        }
    }

    pub fn inline_box() -> Self {
        Self::InlineContents(InlineContents::InlineBox)
    }

    pub fn text_run() -> Self {
        Self::InlineContents(InlineContents::TextRun)
    }
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
                            (OuterDisplayType::Block, InnerDisplayType::Flow) => {
                                BoxData::block_box()
                            }
                            (OuterDisplayType::Inline, InnerDisplayType::Flow)
                            | (OuterDisplayType::Inline, InnerDisplayType::FlowRoot) => {
                                BoxData::inline_box()
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

    pub fn is_root_element(&self) -> bool {
        match &self.node {
            Some(node) => match node.node.as_element_opt() {
                Some(element) => element.tag_name() == "html",
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_body_element(&self) -> bool {
        match &self.node {
            Some(node) => match node.node.as_element_opt() {
                Some(element) => element.tag_name() == "body",
                _ => false,
            },
            _ => false,
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

    pub fn containing_block_opt(&self) -> Option<Rc<LayoutBox>> {
        match self.base.containing_block.borrow().clone() {
            Some(containing_block) => containing_block.upgrade(),
            _ => None,
        }
    }

    pub fn can_have_children(&self) -> bool {
        match self.data {
            BoxData::InlineContents(InlineContents::TextRun) => false,
            _ => true,
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
            BoxData::BlockBox { .. } => true,
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

    pub fn box_model(&self) -> &RefCell<BoxModel> {
        &self.base.box_model
    }

    pub fn content_size(&self) -> Size {
        self.base.content_size.borrow().clone()
    }

    pub fn set_content_width(&self, width: f32) {
        self.base.content_size.borrow_mut().width = width;
    }

    pub fn set_content_height(&self, height: f32) {
        self.base.content_size.borrow_mut().height = height;
    }

    pub fn set_offset(&self, x: f32, y: f32) {
        self.base.offset.borrow_mut().x = x;
        self.base.offset.borrow_mut().y = y;
    }

    pub fn offset(&self) -> Point {
        self.base.offset.borrow().clone()
    }

    pub fn margin_box_height(&self) -> f32 {
        let margin_box = self.base.box_model.borrow().margin_box();
        self.content_size().height + margin_box.top + margin_box.bottom
    }

    pub fn margin_box_width(&self) -> f32 {
        let margin_box = self.base.box_model.borrow().margin_box();
        self.content_size().width + margin_box.left + margin_box.right
    }

    pub fn absolute_rect(&self) -> Rect {
        let mut rect = Rect::from((self.offset(), self.content_size()));

        let mut containing_block = self.containing_block_opt();

        while let Some(block) = containing_block {
            rect.translate(block.offset().x, block.offset().y);
            containing_block = block.containing_block_opt();
        }

        rect
    }

    pub fn absolute_location(&self) -> Point {
        let absolute_rect = self.absolute_rect();
        Point::new(absolute_rect.x, absolute_rect.y)
    }

    pub fn border_box_absolute(&self) -> Rect {
        let border_box = self.base.box_model.borrow().border_box();
        self.padding_box_absolute().add_outer_edges(&border_box)
    }

    pub fn padding_box_absolute(&self) -> Rect {
        let padding_box = self.base.box_model.borrow().padding_box();
        self.absolute_rect().add_outer_edges(&padding_box)
    }

    pub fn render_node(&self) -> Option<Rc<RenderNode>> {
        self.node.clone()
    }

    pub fn friendly_name(&self) -> &str {
        match self.data {
            BoxData::BlockBox { .. } => "BlockBox",
            BoxData::InlineContents(InlineContents::TextRun) => "TextRun",
            BoxData::InlineContents(_) => "InlineBox",
        }
    }

    pub fn formatting_context(&self) -> Rc<dyn FormattingContext> {
        self.base
            .formatting_context
            .borrow()
            .clone()
            .expect("No layout context! This should not happen!")
    }

    pub fn apply_explicit_sizes(&self) {
        let containing_block = self.containing_block().content_size();

        if self.is_inline() && !self.is_inline_block() {
            return;
        }

        if let Some(render_node) = self.render_node() {
            let computed_width = render_node.get_style(&Property::Width);
            let computed_height = render_node.get_style(&Property::Height);

            if !computed_width.is_auto() {
                let used_width = computed_width.to_px(containing_block.width);
                self.set_content_width(used_width);
            }

            if !computed_height.is_auto() {
                let used_height = computed_height.to_px(containing_block.height);
                self.set_content_height(used_height);
            }
        }
    }

    pub fn add_child(parent: Rc<LayoutBox>, child: Rc<LayoutBox>) {
        parent.children_mut().push(child.clone());
        child.set_parent(parent);
    }

    pub fn lines(&self) -> &RefCell<Vec<LineBox>> {
        match &self.data {
            BoxData::BlockBox { lines } => lines,
            _ => unreachable!("Non-block box does not have line boxes"),
        }
    }

    pub fn dump(&self, level: usize) -> String {
        let mut result = String::new();

        let box_type = if self.is_anonymous() {
            format!("[Anonymous {}]", self.friendly_name())
        } else {
            format!("[{}]", self.friendly_name())
        };

        let dimensions = format!(
            " (x: {} | y: {} | w: {} | h: {})",
            self.absolute_rect().x,
            self.absolute_rect().y,
            self.absolute_rect().width,
            self.absolute_rect().height,
        );

        let node_info = match &self.render_node() {
            Some(node) => format!(" {:?}", node.node),
            None => String::new(),
        };

        result.push_str(&format!(
            "{}{}{}{}\n",
            "  ".repeat(level),
            box_type,
            node_info,
            dimensions
        ));

        if self.is_block() && self.children_are_inline() {
            for line in self.lines().borrow().iter() {
                result.push_str(&line.dump(level + 1));
            }
        } else {
            for node in self.children().iter() {
                result.push_str(&node.dump(level + 1));
            }
        }

        return result;
    }
}
