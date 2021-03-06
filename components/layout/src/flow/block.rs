use crate::box_model::{BoxComponent, Edge};
use crate::formatting_context::{apply_explicit_sizes, layout_children, FormattingContext};
use crate::layout_box::LayoutBox;
use style::value_processing::Property;

#[derive(Debug)]
struct BaseFormattingContext {
    pub offset_y: f32,
    pub height: f32,
}

pub struct BlockFormattingContext {
    base: BaseFormattingContext,
    containing_block: *mut LayoutBox,
}

impl BlockFormattingContext {
    pub fn new(layout_box: &mut LayoutBox) -> Self {
        let rect = &layout_box.dimensions.content;

        Self {
            base: BaseFormattingContext {
                offset_y: rect.y,
                height: 0.,
            },
            containing_block: layout_box,
        }
    }

    fn calculate_width(&mut self, layout_box: &mut LayoutBox) {
        let render_node = match &layout_box.render_node {
            Some(node) => node.clone(),
            None => return,
        };

        let containing_block = &self.get_containing_block().dimensions.content;

        let render_node = render_node.borrow();
        let computed_width = render_node.get_style(&Property::Width);
        let computed_margin_left = render_node.get_style(&Property::MarginLeft);
        let computed_margin_right = render_node.get_style(&Property::MarginRight);
        let computed_border_left = render_node.get_style(&Property::BorderLeftWidth);
        let computed_border_right = render_node.get_style(&Property::BorderRightWidth);
        let computed_padding_left = render_node.get_style(&Property::PaddingLeft);
        let computed_padding_right = render_node.get_style(&Property::PaddingRight);
        let containing_width = containing_block.width;

        let box_width = computed_margin_left.to_px(containing_width)
            + computed_border_left.to_px(containing_width)
            + computed_padding_left.to_px(containing_width)
            + computed_width.to_px(containing_width)
            + computed_padding_right.to_px(containing_width)
            + computed_border_right.to_px(containing_width)
            + computed_margin_right.to_px(containing_width);

        let mut used_width = computed_width.to_px(containing_width);
        let mut used_margin_left = computed_margin_left.to_px(containing_width);
        let mut used_margin_right = computed_margin_right.to_px(containing_width);

        // 3. block-level, non-replaced elements in normal flow
        if layout_box.is_non_replaced() {
            if !computed_width.is_auto() && box_width > containing_width {
                if computed_margin_left.is_auto() {
                    used_margin_left = 0.0;
                }
                if computed_margin_right.is_auto() {
                    used_margin_right = 0.0;
                }
            }

            let underflow = containing_width - box_width;

            match (
                computed_width.is_auto(),
                computed_margin_left.is_auto(),
                computed_margin_right.is_auto(),
            ) {
                // If all of the above have a computed value other than 'auto',
                // the values are said to be "over-constrained" and one of the
                // used values will have to be different from its computed value.
                // If the 'direction' property of the containing block has the
                // value 'ltr', the specified value of 'margin-right' is ignored
                // and the value is calculated so as to make the equality true.
                // If the value of 'direction' is 'rtl', this happens to
                // 'margin-left' instead.
                (false, false, false) => {
                    // TODO: support direction rtl
                    used_margin_right = computed_margin_right.to_px(containing_width) + underflow;
                }
                // If there is exactly one value specified as 'auto',
                // its used value follows from the equality.
                (false, true, false) => {
                    used_margin_left = underflow;
                }
                (false, false, true) => {
                    used_margin_right = underflow;
                }
                // If 'width' is set to 'auto', any other 'auto' values become '0'
                // and 'width' follows from the resulting equality.
                (true, _, _) => {
                    if computed_margin_left.is_auto() {
                        used_margin_left = 0.0;
                    }
                    if computed_margin_right.is_auto() {
                        used_margin_right = 0.0;
                    }

                    if underflow >= 0. {
                        used_width = underflow;
                    } else {
                        used_width = 0.;
                        used_margin_right =
                            computed_margin_right.to_px(containing_width) + underflow;
                    }
                }
                // If both 'margin-left' and 'margin-right' are 'auto', their
                // used values are equal. This horizontally centers the element
                // with respect to the edges of the containing block.
                (false, true, true) => {
                    let half_underflow = underflow / 2.;
                    used_margin_left = half_underflow;
                    used_margin_right = half_underflow;
                }
            }
        }

        // apply all calculated used values
        let box_model = layout_box.box_model();
        box_model.set_width(used_width);
        box_model.set(BoxComponent::Margin, Edge::Left, used_margin_left);
        box_model.set(BoxComponent::Margin, Edge::Right, used_margin_right);
        box_model.set(
            BoxComponent::Padding,
            Edge::Left,
            computed_padding_left.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Padding,
            Edge::Right,
            computed_padding_right.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Border,
            Edge::Left,
            computed_border_left.to_px(containing_width),
        );
        box_model.set(
            BoxComponent::Border,
            Edge::Right,
            computed_border_right.to_px(containing_width),
        );
    }

    fn update_new_data(&mut self, layout_box: &LayoutBox) {
        let rect = layout_box.dimensions.margin_box();
        self.base.height += rect.height;
        self.base.offset_y += rect.height;
    }

    fn calculate_position(&mut self, layout_box: &mut LayoutBox) {
        let containing_block = self.get_containing_block();
        let containing_block = &containing_block.dimensions.content.clone();

        let render_node = layout_box.render_node.clone();
        let box_model = layout_box.box_model();

        if let Some(render_node) = render_node {
            let render_node = render_node.borrow();

            let margin_top = render_node
                .get_style(&Property::MarginTop)
                .to_px(containing_block.width);
            let margin_bottom = render_node
                .get_style(&Property::MarginBottom)
                .to_px(containing_block.width);

            let border_top = render_node
                .get_style(&Property::BorderTopWidth)
                .to_px(containing_block.width);
            let border_bottom = render_node
                .get_style(&Property::BorderBottomWidth)
                .to_px(containing_block.width);

            let padding_top = render_node
                .get_style(&Property::PaddingTop)
                .to_px(containing_block.width);
            let padding_bottom = render_node
                .get_style(&Property::PaddingBottom)
                .to_px(containing_block.width);

            box_model.set(BoxComponent::Margin, Edge::Top, margin_top);
            box_model.set(BoxComponent::Margin, Edge::Bottom, margin_bottom);

            box_model.set(BoxComponent::Padding, Edge::Top, padding_top);
            box_model.set(BoxComponent::Padding, Edge::Bottom, padding_bottom);

            box_model.set(BoxComponent::Border, Edge::Top, border_top);
            box_model.set(BoxComponent::Border, Edge::Bottom, border_bottom);
        }

        let content_area_x = containing_block.x
            + box_model.margin.left
            + box_model.border.left
            + box_model.padding.left;

        let content_area_y = self.base.offset_y
            + box_model.margin.top
            + box_model.border.top
            + box_model.padding.top;

        layout_box
            .box_model()
            .set_position(content_area_x, content_area_y);
    }
}

impl FormattingContext for BlockFormattingContext {
    fn layout(&mut self, boxes: Vec<&mut LayoutBox>) -> f32 {
        let containing_block = self.get_containing_block();
        let containing_block = &containing_block.dimensions.content.clone();

        for layout_box in boxes {
            self.calculate_width(layout_box);
            self.calculate_position(layout_box);
            layout_children(layout_box);
            apply_explicit_sizes(layout_box, containing_block);
            self.update_new_data(layout_box);
        }

        self.base.height
    }

    fn get_containing_block(&mut self) -> &mut LayoutBox {
        unsafe { self.containing_block.as_mut().unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout_box::BoxType;
    use crate::tree_builder::*;
    use css::cssom::css_rule::CSSRule;
    use style::build_render_tree;
    use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};
    use test_utils::css::parse_stylesheet;
    use test_utils::dom_creator::*;

    #[test]
    fn test_block_layout_simple() {
        let document = document();
        let dom = element(
            "div",
            document.clone(),
            vec![
                element("div.box", document.clone(), vec![]),
                element("div.box", document.clone(), vec![]),
                element("div.box", document.clone(), vec![]),
                element("div.box", document.clone(), vec![]),
            ],
        );

        let css = r#"
        div {
            display: block;
        }
        .box {
            height: 10px;
        }"#;

        let stylesheet = parse_stylesheet(css);

        let rules = stylesheet
            .iter()
            .map(|rule| match rule {
                CSSRule::Style(style) => ContextualRule {
                    inner: style,
                    location: CSSLocation::Embedded,
                    origin: CascadeOrigin::User,
                },
            })
            .collect::<Vec<ContextualRule>>();

        let render_tree = build_render_tree(dom.clone(), &rules);

        let layout_tree_builder = TreeBuilder::new(render_tree.root.unwrap());

        let layout_box = layout_tree_builder.build();

        let mut layout_box = layout_box.unwrap();

        let mut screen = LayoutBox::new_anonymous(BoxType::Block);

        let mut formatting_context = BlockFormattingContext::new(&mut screen);

        formatting_context.layout(vec![&mut layout_box]);

        //println!("{}", layout_box.dump(&LayoutDumpSpecificity::StructureAndDimensions));

        assert_eq!(formatting_context.base.height, 40.);
        assert_eq!(formatting_context.base.offset_y, 40.);
    }
}
