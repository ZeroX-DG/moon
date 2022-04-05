use std::rc::Rc;

use dom::node::NodePtr;
use layout::dump_layout;
use layout::formatting_context::{establish_context, FormattingContextType};
use layout::{formatting_context::LayoutContext, layout_box::LayoutBox};
use shared::primitive::{Rect, Size};
use style::render_tree::RenderTree;

pub struct Frame {
    document: Option<NodePtr>,
    layout: FrameLayout,
    size: Size,
}

pub struct FrameLayout {
    layout_tree: Option<Rc<LayoutBox>>,
    render_tree: Option<RenderTree>,
}

#[derive(Debug)]
pub enum ReflowType {
    All(NodePtr),
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

    pub fn size(&self) -> Size {
        self.size.clone()
    }

    pub fn resize(&mut self, new_size: Size) {
        self.size = new_size;
        self.layout.reflow(&self.size, ReflowType::LayoutOnly);
    }

    pub fn set_document(&mut self, document: NodePtr) {
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

    pub fn recalculate_styles(&mut self, document_node: NodePtr) {
        let document = document_node.as_document();
        let style_rules = document.style_rules();

        log::debug!("Building render tree");
        self.render_tree = Some(style::tree_builder::TreeBuilder::build(
            document_node,
            &style_rules,
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
                let layout_context = LayoutContext {
                    viewport: Rect {
                        x: 0.,
                        y: 0.,
                        width: size.width,
                        height: size.height,
                    },
                };

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
                    .run(&layout_context, initial_block_box.clone());
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
