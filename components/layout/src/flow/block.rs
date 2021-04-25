use crate::box_model::{BoxComponent, Edge};
use crate::formatting_context::{BaseFormattingContext, FormattingContext};
use crate::layout_box::LayoutBox;
use style::value_processing::Property;

pub struct BlockFormattingContext {
    base: BaseFormattingContext,
    containing_block: *mut LayoutBox,
}

impl BlockFormattingContext {
    pub fn new(layout_box: &mut LayoutBox) -> Self {
        let rect = &layout_box.dimensions.content;

        Self {
            base: BaseFormattingContext {
                offset_x: rect.x,
                offset_y: rect.y,
                width: 0.,
                height: 0.,
            },
            containing_block: layout_box
        }
    }
}

impl FormattingContext for BlockFormattingContext {
    fn base(&self) -> &BaseFormattingContext {
        &self.base
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

        if self.base.width < rect.width {
            self.base.width = rect.width;
        }
    }

    fn get_containing_block(&mut self) -> &mut LayoutBox {
        unsafe {self.containing_block.as_mut().unwrap()}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_builder::*;
    use crate::layout_box::BoxType;
    use css::cssom::css_rule::CSSRule;
    use style::build_render_tree;
    use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};
    use test_utils::css::parse_stylesheet;
    use test_utils::dom_creator::*;

    #[test]
    fn test_block_layout_simple() {
        let dom = element(
            "div",
            vec![
                element("div.box", vec![]),
                element("div.box", vec![]),
                element("div.box", vec![]),
                element("div.box", vec![]),
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

        assert_eq!(formatting_context.base().height, 40.);
        assert_eq!(formatting_context.base().width, 1600.);
        assert_eq!(formatting_context.base().offset_y, 40.);
        assert_eq!(formatting_context.base().offset_x, 0.);
    }
}
