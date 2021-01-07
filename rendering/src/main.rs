mod layout_engine;
mod parsing;

use dom::dom_ref::NodeRef;
use layout::box_model::Rect;
use layout_engine::LayoutEngine;
use clap::{App, Arg, ArgMatches};
use parsing::{parse_css, parse_html};
use std::io::Read;

pub struct Renderer {
    id: String,
    document: Option<NodeRef>,
    layout_engine: LayoutEngine,
}

impl Renderer{
    pub fn new(viewport: Rect) -> Self {
        Self {
            id: nanoid::simple(),
            document: None,
            layout_engine: LayoutEngine::new(viewport),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn load_html(&mut self, input: &mut dyn Read) {
        let mut html = String::new();
        input.read_to_string(&mut html).expect("Cannot read HTML");

        let dom = parse_html(html);

        self.document = Some(dom.clone());

        self.layout_engine.load_dom_tree(&dom);
    }

    pub fn load_css(&mut self, input: &mut dyn Read) {
        let mut css = String::new();
        input.read_to_string(&mut css).expect("Cannot read CSS");

        let style = parse_css(css);

        self.layout_engine.append_stylesheet(style);
    }
}

fn init_logging(id: &str) {
    let mut log_dir = dirs::home_dir().expect("Home directory not found");
    log_dir.push("/tmp/moon");
    std::fs::create_dir_all(&log_dir).expect("Cannot create log directory");

    log_dir.push(format!("renderer_log.txt"));
    simple_logging::log_to_file(log_dir, log::LevelFilter::Debug).expect("Can not open log file");
}

fn accept_cli<'a>() -> ArgMatches<'a> {
    App::new("Moon Renderer")
        .version("1.0")
        .author("Viet-Hung Nguyen <viethungax@gmail.com>")
        .about("Renderer for moon browser")
        .arg(
            Arg::with_name("html")
                .required(true)
                .long("html")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("css")
                .required(true)
                .long("css")
                .takes_value(true),
        )
        .get_matches()
}

fn main() {
    let matches = accept_cli();
    let viewport = Rect {
        x: 0.,
        y: 0.,
        width: 500.,
        height: 300.
    };
    let mut renderer = Renderer::new(viewport);

    if let Some(html_path) = matches.value_of("html") {
        let mut html_file = std::fs::File::open(html_path).expect("Unable to open HTML file");
        renderer.load_html(&mut html_file);
    }

    if let Some(css_path) = matches.value_of("css") {
        let mut css_file = std::fs::File::open(css_path).expect("Unable to open CSS file");
        renderer.load_css(&mut css_file);
    }

    init_logging(renderer.id());
}
