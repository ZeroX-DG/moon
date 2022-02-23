use std::rc::Rc;

use css::cssom::css_rule::CSSRule;

use dom::node::Node;
use layout::dump_layout;
use layout::formatting_context::{establish_context, FormattingContextType};
use layout::{formatting_context::LayoutContext, layout_box::LayoutBox};
use shared::primitive::{Rect, Size};
use style::render_tree::RenderTree;
use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};

pub struct Frame {
    document: Option<Rc<Node>>,
    layout: FrameLayout,
    size: Size,
}

pub struct FrameLayout {
    layout_tree: Option<Rc<LayoutBox>>,
    render_tree: Option<RenderTree>,
}

#[derive(Debug)]
pub enum ReflowType {
    All(Rc<Node>),
    LayoutOnly,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            document: None,
            layout: FrameLayout::new(),
            size: Size::new(0., 0.),
        }
    }

    pub fn resize(&mut self, new_size: Size) {
        self.size = new_size;
        self.layout.reflow(&self.size, ReflowType::LayoutOnly);
    }

    pub fn set_document(&mut self, document: Rc<Node>) {
        self.document = Some(document.clone());
        self.layout.reflow(&self.size, ReflowType::All(document));
    }

    pub fn layout(&self) -> &FrameLayout {
        &self.layout
    }
}

impl FrameLayout {
    pub fn new() -> Self {
        Self {
            layout_tree: None,
            render_tree: None,
        }
    }

    pub fn layout_tree(&self) -> Option<Rc<LayoutBox>> {
        self.layout_tree.clone()
    }

    pub fn recalculate_styles(&mut self, document_node: Rc<Node>) {
        let document = document_node.as_document();
        let stylesheets = document.stylesheets();
        // TODO: cache this step so we don't have to flat map on every reflow
        let contextual_rules: Vec<ContextualRule> = stylesheets
            .iter()
            .flat_map(|stylesheet| {
                stylesheet.iter().map(|rule| match rule {
                    CSSRule::Style(style) => ContextualRule {
                        inner: style,
                        location: CSSLocation::Embedded,
                        origin: CascadeOrigin::User,
                    },
                })
            })
            .collect();

        log::debug!("Building render tree");
        self.render_tree = Some(style::tree_builder::TreeBuilder::build(
            document_node,
            &contextual_rules,
        ));
        log::debug!("Finished render tree");
    }

    pub fn recalculate_layout(&mut self, size: &Size) {
        if let Some(render_tree) = &self.render_tree {
            log::debug!("Building layout tree");
            let render_tree_root = render_tree.root.clone().unwrap();
            self.layout_tree =
                Some(layout::tree_builder::TreeBuilder::new().build(render_tree_root));
            log::debug!("Finished layout tree");

            if let Some(root) = &self.layout_tree {
                log::debug!("Starting layout process");
                let layout_context = Rc::new(LayoutContext {
                    viewport: Rect {
                        x: 0.,
                        y: 0.,
                        width: size.width,
                        height: size.height,
                    },
                });

                let initial_block_box = Rc::new(LayoutBox::new_anonymous(
                    layout::layout_box::BoxData::block_box(),
                ));
                LayoutBox::add_child(initial_block_box.clone(), root.clone());

                establish_context(
                    FormattingContextType::BlockFormattingContext,
                    initial_block_box.clone(),
                );
                initial_block_box
                    .formatting_context()
                    .run(layout_context.clone(), initial_block_box.clone());
                log::debug!("Finished layout process");
                dump_layout!(root.clone());
            } else {
                log::info!("Empty layout tree. Skipping layout process");
            }
        }
    }

    pub fn reflow(&mut self, size: &Size, type_: ReflowType) {
        log::debug!("Start reflowing with type: {:?}", type_);
        match &type_ {
            ReflowType::LayoutOnly => {
                self.recalculate_layout(size);
            }
            ReflowType::All(document) => {
                self.recalculate_styles(document.clone());
                self.recalculate_layout(size);
            }
        }
        log::debug!("Finished reflowing with type: {:?}", type_);
    }
}
