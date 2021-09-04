use super::loader::frame::FrameLoader;
use css::cssom::css_rule::CSSRule;
use dom::dom_ref::NodeRef;

use layout::{
    flow::block::{BlockBox, BlockFormattingContext},
    formatting_context::{FormattingContext, LayoutContext},
    layout_box::{LayoutNodeId, LayoutTree},
};
use style::render_tree::RenderTree;
use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};
use shared::primitive::Rect;

pub type FrameSize = (u32, u32);

pub struct Frame {
    document: Option<NodeRef>,
    layout: FrameLayout,
    size: FrameSize,
}

pub struct FrameLayout {
    layout_tree: LayoutTree,
    render_tree: Option<RenderTree>,
}

#[derive(Debug)]
pub enum ReflowType {
    All(NodeRef),
    LayoutOnly,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            document: None,
            layout: FrameLayout::new(),
            size: (0, 0),
        }
    }

    pub fn resize(&mut self, new_size: FrameSize) {
        self.size = new_size;
        self.layout.reflow(self.size, ReflowType::LayoutOnly);
    }

    pub fn size(&self) -> FrameSize {
        self.size.clone()
    }

    pub fn set_document(&mut self, document: NodeRef) {
        self.document = Some(document.clone());
        self.layout.reflow(self.size, ReflowType::All(document));
    }

    pub fn load_html(&mut self, html: String) {
        self.set_document(FrameLoader::load_html(html));
    }

    pub fn layout(&self) -> &FrameLayout {
        &self.layout
    }
}

impl FrameLayout {
    pub fn new() -> Self {
        Self {
            layout_tree: LayoutTree::new(),
            render_tree: None,
        }
    }

    pub fn root(&self) -> Option<LayoutNodeId> {
        self.layout_tree.root()
    }

    pub fn layout_tree(&self) -> &LayoutTree {
        &self.layout_tree
    }

    pub fn recalculate_styles(&mut self, document: NodeRef) {
        let document_clone = document.clone();
        let document_borrow = document_clone.borrow();
        let document_borrow = document_borrow.as_document();
        let stylesheets = document_borrow.stylesheets();
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
            document,
            &contextual_rules,
        ));
        log::debug!("Finished render tree");
    }

    pub fn recalculate_layout(&mut self, size: FrameSize) {
        if let Some(render_tree) = &self.render_tree {
            log::debug!("Building layout tree");
            self.layout_tree =
                layout::tree_builder::TreeBuilder::new().build(render_tree.root.clone().unwrap());
            log::debug!("Finished layout tree");

            if let Some(root) = self.layout_tree.root() {
                let (width, height) = size;

                let layout_context = LayoutContext {
                    viewport: Rect {
                        x: 0.,
                        y: 0.,
                        width: width as f32,
                        height: height as f32,
                    },
                };

                let initial_block_box = self
                    .layout_tree
                    .set_root(Box::new(BlockBox::new_anonymous()));
                self.layout_tree.add_child_by_id(&initial_block_box, &root);

                let mut formatting_context =
                    BlockFormattingContext::new(&layout_context, &mut self.layout_tree);

                formatting_context.run(&initial_block_box);
            }
        }
    }

    pub fn reflow(&mut self, size: FrameSize, type_: ReflowType) {
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
