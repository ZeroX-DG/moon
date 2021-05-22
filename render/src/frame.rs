use dom::dom_ref::NodeRef;
use css::cssom::stylesheet::StyleSheet;
use css::cssom::css_rule::CSSRule;
use super::loader::frame::FrameLoader;
use super::loader::css::CSSLoader;
use super::paint::{Painter, OutputBitmap};

use layout::{build_layout_tree, layout_box::LayoutBox, box_model::Rect};
use style::render_tree::{build_render_tree, RenderTree};
use style::value_processing::{ContextualRule, CSSLocation, CascadeOrigin};

pub struct Frame {
    document: Option<NodeRef>,
    stylesheets: Vec<StyleSheet>,
    layout: FrameLayout,
    size: (u32, u32),
}

pub struct FrameLayout {
    layout_tree: Option<LayoutBox>,
    render_tree: Option<RenderTree>,
}

pub enum ReflowType<'a> {
    All(NodeRef, &'a [StyleSheet]),
    LayoutOnly
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

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        self.layout.reflow(self.size, ReflowType::LayoutOnly);
    }

    pub fn append_stylesheet(&mut self, stylesheet: StyleSheet) {
        self.stylesheets.push(stylesheet);
        
        if let Some(document) = &self.document {
            self.layout.reflow(self.size, ReflowType::All(document.clone(), &self.stylesheets));
        }
    }

    pub fn set_document(&mut self, document: NodeRef) {
        self.document = Some(document.clone());
        self.layout.reflow(self.size, ReflowType::All(document, &self.stylesheets));
    }

    pub fn load_html(&mut self, html: String) {
        self.set_document(FrameLoader::load_html(html));
    }

    pub fn load_css(&mut self, css: String) {
        self.append_stylesheet(CSSLoader::load_from_text(css));
    }

    pub async fn paint(&self, painter: &mut Painter) -> Option<OutputBitmap> {
        if let Some(layout_tree) = &self.layout.layout_tree {
            let display_list = painting::build_display_list(layout_tree);
            painting::paint(&display_list, painter);

            return painter.paint(self.size).await;
        }

        None
    }
}

impl FrameLayout {
    pub fn new() -> Self {
        Self {
            layout_tree: None,
            render_tree: None,
        }
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
                    }
                })
            })
            .collect();

        self.render_tree = Some(build_render_tree(document, &contextual_rules));
    }

    pub fn recalculate_layout(&mut self, size: (u32, u32)) {
        if let Some(render_tree) = &self.render_tree {
            self.layout_tree = build_layout_tree(render_tree);

            if let Some(layout_tree) = &mut self.layout_tree {
                let (width, height) = size;

                layout::compute_layout(layout_tree, &Rect {
                    x: 0.,
                    y: 0.,
                    width: width as f32,
                    height: height as f32
                });
            }
        }
    }

    pub fn reflow(&mut self, size: (u32, u32), type_: ReflowType) {
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
