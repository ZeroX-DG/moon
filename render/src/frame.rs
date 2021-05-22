use super::loader::css::CSSLoader;
use super::loader::frame::FrameLoader;
use css::cssom::css_rule::CSSRule;
use css::cssom::stylesheet::StyleSheet;
use dom::dom_ref::NodeRef;

use layout::{box_model::Rect, build_layout_tree, layout_box::LayoutBox};
use style::render_tree::{build_render_tree, RenderTree};
use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};

pub type FrameSize = (u32, u32);

pub struct Frame {
    document: Option<NodeRef>,
    stylesheets: Vec<StyleSheet>,
    layout: FrameLayout,
    size: FrameSize,
}

pub struct FrameLayout {
    layout_tree: Option<LayoutBox>,
    render_tree: Option<RenderTree>,
}

pub enum ReflowType<'a> {
    All(NodeRef, &'a [StyleSheet]),
    LayoutOnly,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            document: None,
            stylesheets: Vec::new(),
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

    pub fn append_stylesheet(&mut self, stylesheet: StyleSheet) {
        self.stylesheets.push(stylesheet);

        if let Some(document) = &self.document {
            self.layout.reflow(
                self.size,
                ReflowType::All(document.clone(), &self.stylesheets),
            );
        }
    }

    pub fn set_document(&mut self, document: NodeRef) {
        self.document = Some(document.clone());
        self.layout
            .reflow(self.size, ReflowType::All(document, &self.stylesheets));
    }

    pub fn load_html(&mut self, html: String) {
        self.set_document(FrameLoader::load_html(html));
    }

    pub fn load_css(&mut self, css: String) {
        self.append_stylesheet(CSSLoader::load_from_text(css));
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

    pub fn root(&self) -> &Option<LayoutBox> {
        &self.layout_tree
    }

    pub fn recalculate_styles(&mut self, document: NodeRef, stylesheets: &[StyleSheet]) {
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

        self.render_tree = Some(build_render_tree(document, &contextual_rules));
    }

    pub fn recalculate_layout(&mut self, size: FrameSize) {
        if let Some(render_tree) = &self.render_tree {
            self.layout_tree = build_layout_tree(render_tree);

            if let Some(layout_tree) = &mut self.layout_tree {
                let (width, height) = size;

                layout::compute_layout(
                    layout_tree,
                    &Rect {
                        x: 0.,
                        y: 0.,
                        width: width as f32,
                        height: height as f32,
                    },
                );
            }
        }
    }

    pub fn reflow(&mut self, size: FrameSize, type_: ReflowType) {
        match type_ {
            ReflowType::LayoutOnly => {
                if self.render_tree.is_none() {
                    log::warn!("FrameLayout: Reflowing with empty render tree!");
                }
                self.recalculate_layout(size);
            }
            ReflowType::All(document, stylesheets) => {
                self.recalculate_styles(document, stylesheets);
                self.recalculate_layout(size);
            }
        }
    }
}
