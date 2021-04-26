use super::layout_box::LayoutBox;
use super::line_box::LineBox;

pub struct LineFragmenter;

impl LineFragmenter {
    pub fn run(layout_box: &mut LayoutBox, containing_block: &mut LayoutBox) {
        let containing_box = containing_block.dimensions.content_box();
        if containing_block.line_boxes.is_empty() {
            containing_block.line_boxes.push(
                LineBox::new(containing_box.x, containing_box.y)
            );
        }

        if let Some(line_box) = containing_block.line_boxes.last() {
            let box_width = layout_box.dimensions.margin_box().width;
            let containing_width = containing_box.width;

            let line_box_rect = line_box.get_rect();

            if box_width + line_box_rect.width > containing_width {
                containing_block.line_boxes.push(
                    LineBox::new(containing_box.x, line_box_rect.y + line_box_rect.height)
                );
            }
            containing_block.line_boxes.last_mut().unwrap().push(layout_box);
        }
    }
}
