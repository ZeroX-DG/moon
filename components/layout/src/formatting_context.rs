use super::layout_box::LayoutBox;
use super::layout::layout;
use super::box_model::Rect;

/// Formatting context of each box
#[derive(Debug, Clone, PartialEq)]
pub struct FormattingContext {
    pub type_: FormattingContextType,
    pub data: FormattingContextData,
    pub containing_block: Rect
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContextType {
    Inline,
    Block,
    Flex,
    Grid
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormattingContextData {
    pub offset_x: f32,
    pub offset_y: f32,
    pub width: f32,
    pub height: f32,
}

impl FormattingContext {
    pub fn new(type_: FormattingContextType, established_box: &LayoutBox) -> Self {
        Self {
            type_,
            data: FormattingContextData {
                offset_x: established_box.dimensions.content.x,
                offset_y: established_box.dimensions.content.y,
                width: established_box.dimensions.content.width,
                height: established_box.dimensions.content.height
            },
            containing_block: established_box.dimensions.content.clone()
        }
    }

    pub fn layout_child(&mut self, child: &mut LayoutBox) {
        layout(child, &self.containing_block, &self.data);
        self.update_each_layout(child);
    }

    fn update_each_layout(&mut self, child: &LayoutBox) {
        match self.type_ {
            FormattingContextType::Inline => self.update_inline_layout(child),
            FormattingContextType::Block => self.update_block_layout(child),
            _ => unimplemented!("Unsupported context: {:#?}", self.type_)
        }
    }

    fn update_inline_layout(&mut self, child: &LayoutBox) {
        let rect = child.dimensions.margin_box();
        self.data.width += rect.width;
        self.data.offset_x += rect.width - child.dimensions.margin.right;

        if self.data.height < rect.height {
            self.data.height = rect.height;
        }
    }

    fn update_block_layout(&mut self, child: &LayoutBox) {
        let rect = child.dimensions.margin_box();
        self.data.height += rect.height;
        self.data.offset_y += rect.height - child.dimensions.margin.bottom;

        if self.data.width < rect.width {
            self.data.width = rect.width;
        }
    }
}