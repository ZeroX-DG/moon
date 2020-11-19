mod painter;

use dom::dom_ref::NodeRef;
use css::cssom::{css_rule::CSSRule, stylesheet::StyleSheet};
use style::{
    value_processing::{ContextualRule, CSSLocation, CascadeOrigin},
    render_tree::build_render_tree
};
use layout::{build_layout_tree, ContainingBlock, layout_box::LayoutBox};
use painting::paint;
use painter::SkiaPainter;
use skulpin::{winit, skia_safe::{Surface, ISize}};
use ipc::IpcRenderer;
use message::MoonMessage;

pub struct Renderer {
    surface: Surface
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            surface: Surface::new_raster_n32_premul(ISize::new(500, 300)).unwrap()
        }
    }

    pub fn draw(&mut self, layout: &LayoutBox, painter: &mut SkiaPainter) {
        let canvas = self.surface.canvas();
        paint(layout, painter, canvas);
    }

    pub fn snapshot(&mut self) -> Vec<u8> {
        let data = self.surface.image_snapshot()
            .encoded_data();
        if let Some(d) = data {
            d.as_bytes().to_vec()
        } else {
            vec![]
        }
    }
}

fn parse_html() -> NodeRef {
    let html = include_str!("../fixtures/test.html");
    let tokenizer = html::tokenizer::Tokenizer::new(html.to_owned());
    let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer);
    let document = tree_builder.run();
    document
}

fn parse_css() -> StyleSheet {
    let css = r#"
        * { display: block; }
        div { padding: 12px; }
        .a { background-color: rgb(52, 152, 219); }
        .b { background-color: rgb(155, 89, 182); }
        .c { background-color: rgb(52, 73, 94); }
        .d { background-color: rgb(231, 76, 60); }
        .e { background-color: rgb(230, 126, 34); }
        .f { background-color: rgb(241, 196, 15); }
        .g { background-color: rgb(64, 64, 122); }
    "#;

    let tokenizer = css::tokenizer::Tokenizer::new(css.to_string());
    let mut parser = css::parser::Parser::<css::tokenizer::token::Token>::new(tokenizer.run());
    let stylesheet = parser.parse_a_css_stylesheet();

    stylesheet
}

fn main() {
    let document = parse_html();
    let stylesheet = parse_css();
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
    // layout
    let render_tree = build_render_tree(document, &rules);
    let mut layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

    let logical_size = winit::dpi::LogicalSize::new(500.0, 300.0);

    layout::layout(&mut layout_tree, &mut ContainingBlock {
        offset_x: 0.,
        offset_y: 0.,
        x: 0.,
        y: 0.,
        width: logical_size.width,
        height: logical_size.height,
        previous_margin_bottom: 0.0,
        collapsed_margins_vertical: 0.0
    });

    let mut painter = SkiaPainter::new();

    let mut renderer = Renderer::new();

    renderer.draw(&layout_tree, &mut painter);

    let ipc_address = std::env::args().skip(1).next().expect("Expected address");
    let ipc = IpcRenderer::<MoonMessage>::new(&ipc_address);

    println!("{}", ipc_address);

    ipc.client.sender.send(MoonMessage::RePaint(renderer.snapshot()))
        .expect("Failed to send canvas");
}
