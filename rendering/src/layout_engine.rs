use css::cssom::css_rule::CSSRule;
use css::cssom::stylesheet::StyleSheet;
use dom::dom_ref::NodeRef;
use layout::{box_model::Rect, build_layout_tree, layout_box::LayoutBox};
use style::render_tree::{build_render_tree, RenderTree};
use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};

pub struct LayoutEngine {
    dom: Option<NodeRef>,
    layout_tree: Option<LayoutBox>,
    render_tree: Option<RenderTree>,
    stylesheets: Vec<StyleSheet>,
    viewport: Rect,
}

impl LayoutEngine {
    pub fn new(viewport: Rect) -> Self {
        Self {
            dom: None,
            layout_tree: None,
            render_tree: None,
            stylesheets: Vec::new(),
            viewport,
        }
    }

    pub fn load_dom_tree(&mut self, dom: &NodeRef) {
        self.dom = Some(dom.clone());
        self.recalculate_styles();
        self.recalculate_layout();
    }

    pub fn append_stylesheet(&mut self, stylesheet: StyleSheet) {
        self.stylesheets.push(stylesheet);
        self.recalculate_styles();
        self.recalculate_layout();
    }

    pub fn recalculate_styles(&mut self) {
        if let Some(dom) = &self.dom {
            let style_rules = self.stylesheets.iter().fold(vec![], |mut acc, stylesheet| {
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
                acc.extend(rules);
                acc
            });
            self.render_tree = Some(build_render_tree(dom.clone(), &style_rules));
        }
    }

    pub fn recalculate_layout(&mut self) {
        if let Some(render_tree) = &self.render_tree {
            self.layout_tree = build_layout_tree(render_tree);

            if let Some(layout_tree) = &mut self.layout_tree {
                layout::compute_layout(layout_tree, &self.viewport);
            }
        }
    }

    pub fn layout_tree(&self) -> &Option<LayoutBox> {
        &self.layout_tree
    }
}
