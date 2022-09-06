use std::{cell::RefCell, fmt::Debug, ops::Deref, rc::Rc};

use dom::node::NodePtr;
use shared::{
    primitive::{Point, Rect, Size},
    tree_node::{TreeNode, TreeNodeHooks},
};
use style_types::{
    values::{
        display::Display,
        display::{InnerDisplayType, OuterDisplayType},
        prelude::Position,
    },
    Property, Value,
};

use crate::{
    box_model::BoxModel,
    flow::line_box::LineBox,
    formatting_context::{FormattingContext, FormattingContextType},
};

#[derive(Debug)]
pub struct LayoutBox {
    pub data: BoxData,
    pub node: Option<NodePtr>,
    pub box_model: RefCell<BoxModel>,
    pub offset: RefCell<Point>,
    pub content_size: RefCell<Size>,
    pub formatting_context: RefCell<Option<Rc<dyn FormattingContext>>>,
    pub scroll_top: RefCell<f32>,
    pub scroll_height: RefCell<f32>,
}

pub struct LayoutBoxPtr(pub TreeNode<LayoutBox>);

impl TreeNodeHooks<LayoutBox> for LayoutBox {}
impl Debug for LayoutBoxPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl Deref for LayoutBoxPtr {
    type Target = TreeNode<LayoutBox>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Clone for LayoutBoxPtr {
    fn clone(&self) -> Self {
        LayoutBoxPtr(self.0.clone())
    }
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
    pub fn new(node: NodePtr) -> Self {
        let box_data = {
            if node.is_text() {
                BoxData::InlineContents(InlineContents::TextRun)
            } else {
                match node.get_style(&Property::Display) {
                    Value::Display(d) => match d {
                        Display::Full(ref outer, ref inner) => match (outer, inner) {
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
            box_model: Default::default(),
            offset: Default::default(),
            content_size: Default::default(),
            scroll_top: RefCell::new(0.),
            scroll_height: RefCell::new(0.),
            formatting_context: RefCell::new(None),
            data: box_data,
            node: Some(node),
        }
    }

    pub fn new_anonymous(data: BoxData) -> Self {
        Self {
            box_model: Default::default(),
            offset: Default::default(),
            scroll_top: RefCell::new(0.),
            scroll_height: RefCell::new(0.),
            content_size: Default::default(),
            formatting_context: RefCell::new(None),
            data,
            node: None,
        }
    }
}

impl LayoutBoxPtr {
    pub fn is_root_element(&self) -> bool {
        match &self.node {
            Some(node) => match node.as_element_opt() {
                Some(element) => element.tag_name() == "html",
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_body_element(&self) -> bool {
        match &self.node {
            Some(node) => match node.as_element_opt() {
                Some(element) => element.tag_name() == "body",
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.node.is_none()
    }

    pub fn children_are_inline(&self) -> bool {
        self.iterate_children()
            .all(|child| LayoutBoxPtr(child).is_inline())
    }

    pub fn is_block_container(&self) -> bool {
        let is_block = !self.children_are_inline();
        let is_inline_block = self.children_are_inline()
            && match self.formatting_context.borrow().deref() {
                Some(context) => {
                    context.base().context_type == FormattingContextType::InlineFormattingContext
                }
                _ => false,
            };

        is_block || is_inline_block
    }

    pub fn containing_block(&self) -> Option<LayoutBoxPtr> {
        if self.is_positioned(Position::Static) || self.is_positioned(Position::Relative) {
            return self
                .find_first_ancestor(|parent| {
                    let parent = LayoutBoxPtr(parent);
                    parent.is_block_container() || parent.formatting_context.borrow().is_some()
                })
                .map(|node| LayoutBoxPtr(node));
        }

        if self.is_positioned(Position::Absolute) {
            return self
                .find_first_ancestor(|parent| !LayoutBoxPtr(parent).is_positioned(Position::Static))
                .map(|node| LayoutBoxPtr(node));
        }

        if self.is_positioned(Position::Fixed) {
            return self
                .find_first_ancestor(|parent| parent.parent().is_none())
                .map(|node| LayoutBoxPtr(node));
        }

        return self
            .find_first_ancestor(|parent| LayoutBoxPtr(parent).is_block_container())
            .map(|node| LayoutBoxPtr(node));
    }

    // TODO: Get parent base on overflow property instead
    pub fn scrolling_containing_block(&self) -> Option<LayoutBoxPtr> {
        self.find_first_ancestor(|parent| parent.parent().is_none())
            .map(|node| LayoutBoxPtr(node))
    }

    pub fn is_visible_in_scrolling_area(&self) -> bool {
        self.scrolling_containing_block()
            .map(|containing_block| {
                self.padding_box_absolute()
                    .is_overlap_rect(&containing_block.absolute_rect())
            })
            .unwrap_or(true)
    }

    // TODO: Support dynamic scroll bar width
    pub fn scrollbar_width(&self) -> f32 {
        12.
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
        match self.node() {
            Some(node) => match node.get_style(&Property::Display) {
                Value::Display(Display::Full(_, InnerDisplayType::FlowRoot)) => self.is_inline(),
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_positioned(&self, position: Position) -> bool {
        match self.node() {
            Some(node) => match node.get_style(&Property::Position) {
                Value::Position(pos) => pos == position,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_non_replaced(&self) -> bool {
        match &self.node() {
            Some(node) => match node.as_element_opt() {
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
        &self.box_model
    }

    pub fn content_size(&self) -> Size {
        self.content_size.borrow().clone()
    }

    pub fn set_content_width(&self, width: f32) {
        self.content_size.borrow_mut().width = width;
    }

    pub fn set_content_height(&self, height: f32) {
        self.content_size.borrow_mut().height = height;
    }

    pub fn set_offset(&self, x: f32, y: f32) {
        self.offset.borrow_mut().x = x;
        self.offset.borrow_mut().y = y;
    }

    pub fn offset(&self) -> Point {
        self.offset.borrow().clone()
    }

    pub fn scroll_top(&self) -> f32 {
        *self.scroll_top.borrow()
    }

    pub fn set_scroll_top(&self, y: f32) {
        *self.scroll_top.borrow_mut() = y;
    }

    pub fn scroll_height(&self) -> f32 {
        *self.scroll_height.borrow()
    }

    pub fn set_scroll_height(&self, height: f32) {
        *self.scroll_height.borrow_mut() = height;
    }

    pub fn scroll(&self, delta_y: f32) -> bool {
        if !self.scrollable() {
            return false;
        }

        self.set_scroll_top(self.scroll_top() + delta_y);

        if self.scroll_top() < 0. {
            self.set_scroll_top(0.);
        }

        let max_scroll_y = self.scroll_height() - self.content_size().height;

        if self.scroll_top() > max_scroll_y {
            self.set_scroll_top(max_scroll_y);
        }

        true
    }

    pub fn scrollable(&self) -> bool {
        self.scroll_height() - self.content_size().height > 0.
    }

    pub fn margin_box_height(&self) -> f32 {
        let margin_box = self.box_model.borrow().margin_box();
        self.content_size().height + margin_box.top + margin_box.bottom
    }

    pub fn margin_box_width(&self) -> f32 {
        let margin_box = self.box_model.borrow().margin_box();
        self.content_size().width + margin_box.left + margin_box.right
    }

    pub fn absolute_rect(&self) -> Rect {
        let mut rect = Rect::from((self.offset(), self.content_size()));

        let mut containing_block = self.containing_block();

        while let Some(block) = containing_block {
            rect.translate(block.offset().x, block.offset().y);
            rect.translate(0., -block.scroll_top());
            containing_block = block.containing_block();
        }

        rect
    }

    pub fn absolute_location(&self) -> Point {
        let absolute_rect = self.absolute_rect();
        Point::new(absolute_rect.x, absolute_rect.y)
    }

    pub fn border_box_absolute(&self) -> Rect {
        let border_box = self.box_model.borrow().border_box();
        self.padding_box_absolute().add_outer_edges(&border_box)
    }

    pub fn padding_box_absolute(&self) -> Rect {
        let padding_box = self.box_model.borrow().padding_box();
        self.absolute_rect().add_outer_edges(&padding_box)
    }

    pub fn node(&self) -> Option<NodePtr> {
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
        self.formatting_context.borrow().clone().unwrap_or_else(|| {
            self.parent()
                .map(|parent| LayoutBoxPtr(parent).formatting_context())
                .expect("Unable to obtain formatting context")
        })
    }

    pub fn apply_explicit_sizes(&self) {
        let containing_block = self.containing_block().unwrap().content_size();

        if self.is_inline() && !self.is_inline_block() {
            return;
        }

        // Set scroll height to the actual content size before the content size get cut off by explicit size.
        self.set_scroll_height(self.content_size().height);

        if let Some(node) = self.node() {
            let computed_width = node.get_style(&Property::Width);
            let computed_height = node.get_style(&Property::Height);

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

    pub fn lines(&self) -> &RefCell<Vec<LineBox>> {
        match &self.data {
            BoxData::BlockBox { lines } => lines,
            _ => unreachable!("Non-block box does not have line boxes"),
        }
    }

    pub fn get_non_anonymous_parent(&self) -> LayoutBoxPtr {
        let mut result = self.clone();

        loop {
            if result.node.is_some() {
                break;
            }
            result = LayoutBoxPtr(result.parent().unwrap());
        }

        return result;
    }

    pub fn dump(&self, level: usize) -> String {
        let mut result = String::new();

        let box_type = if self.is_anonymous() {
            format!("[Anonymous {}]", self.friendly_name())
        } else {
            format!("[{}]", self.friendly_name())
        };

        let formatting_context = format!("[{:?}]", self.formatting_context().base().context_type);

        let dimensions = format!(
            " (x: {} | y: {} | w: {} | h: {})",
            self.absolute_rect().x,
            self.absolute_rect().y,
            self.absolute_rect().width,
            self.absolute_rect().height,
        );

        let node_info = match &self.node() {
            Some(node) => format!(" {:?}", node),
            None => String::new(),
        };

        result.push_str(&format!(
            "{}{}{}{}{}\n",
            "  ".repeat(level),
            box_type,
            formatting_context,
            node_info,
            dimensions
        ));

        if self.is_block() && self.children_are_inline() {
            for line in self.lines().borrow().iter() {
                result.push_str(&line.dump(level + 1));
            }
        } else {
            self.for_each_child(|node| {
                result.push_str(&LayoutBoxPtr(node).dump(level + 1));
            });
        }

        return result;
    }
}
