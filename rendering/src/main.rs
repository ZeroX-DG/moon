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
use skia_safe::{Surface, ISize};
use ipc::{Client, Sender};
use std::io::{self, Stdin, StdinLock, Stdout, StdoutLock};
use message::{KernelMessage, RendererMessage};

use lazy_static::lazy_static;

lazy_static! {
    static ref STDIN: Stdin = io::stdin();
    static ref STDOUT: Stdout = io::stdout();
}

pub fn stdinlock() -> StdinLock<'static> {
    STDIN.lock()
}

pub fn stdoutlock() -> StdoutLock<'static> {
    STDOUT.lock()
}

pub struct Renderer<'a> {
    id: String,
    surface: Surface,
    sender: &'a mut Sender<RendererMessage>
}

impl<'a> Renderer<'a> {
    pub fn new(sender: &'a mut Sender<RendererMessage>) -> Self {
        Self {
            id: nanoid::simple(),
            surface: Surface::new_raster_n32_premul(ISize::new(500, 300)).unwrap(),
            sender
        }
    }

    pub fn draw(&mut self, layout: &LayoutBox, painter: &mut SkiaPainter) {
        let canvas = self.surface.canvas();
        paint(layout, painter, canvas);
    }

    pub fn handle_kernel_msg(&mut self, msg: KernelMessage) {
        match msg {
            KernelMessage::LoadUrl(url) => self.load_url(url),
            _ => {
                log::debug!("Unknown kernel message: {:?}", msg);
            }
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn load_url(&mut self, url: String) {
        self.sender.send(RendererMessage::SetTitle("Hello!".to_string()))
            .expect("Can't send message to kernel");
    }
}

fn init_logging(id: &str) {
    let mut log_dir = dirs::home_dir().expect("Home directory not found");
    log_dir.push("/tmp/moon");
    std::fs::create_dir_all(&log_dir).expect("Cannot create log directory");

    log_dir.push(format!("renderer_{}_log.txt", id));
    simple_logging::log_to_file(log_dir, log::LevelFilter::Debug)
        .expect("Can not open log file");
}

fn main() {
    let mut ipc = Client::<KernelMessage, RendererMessage>::new(stdinlock, stdoutlock);
    let mut renderer = Renderer::new(&mut ipc.sender);

    init_logging(renderer.id());

    loop {
        match ipc.receiver.recv() {
            Ok(msg) => {
                if let KernelMessage::Exit = msg {
                    log::info!("Received exit. Goodbye!");
                    break
                }
                renderer.handle_kernel_msg(msg);
            },
            Err(_) => break
        }
    }

    ipc.close().expect("Unable to close Ipc");
}

