mod layout_engine;
mod parsing;

use dom::dom_ref::NodeRef;
use layout::box_model::Rect;
use layout::layout_printer::{layout_to_string, DumpSpecificity};
use layout_engine::LayoutEngine;

use ipc::{Client, Sender};
use message::{KernelMessage, RendererMessage};
use std::fs;
use std::io::{self, Stdin, StdinLock, Stdout, StdoutLock};

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
    sender: &'a mut Sender<RendererMessage>,
    document: Option<NodeRef>,
    layout_engine: LayoutEngine,
}

impl<'a> Renderer<'a> {
    pub fn new(sender: &'a mut Sender<RendererMessage>) -> Self {
        Self {
            id: nanoid::simple(),
            sender,
            document: None,
            layout_engine: LayoutEngine::new(Rect {
                x: 0.,
                y: 0.,
                width: 500.,
                height: 300.,
            }),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn handle_kernel_msg(&mut self, msg: KernelMessage) {
        match msg {
            KernelMessage::LoadHTMLLocal(path) => {
                self.load_html_local(path);
                self.repaint();
            }
            KernelMessage::LoadCSSLocal(path) => {
                self.load_css_local(path);
                self.repaint();
            }
            _ => {
                log::debug!("Unknown kernel message: {:?}", msg);
            }
        }
    }

    fn repaint(&mut self) {
        if let Some(layout_tree) = self.layout_engine.layout_tree() {
            log::debug!(
                "Generated layout box tree:\n {}",
                layout_to_string(layout_tree, 0, &DumpSpecificity::StructureAndDimensions)
            );
            let display_list = painting::build_display_list(layout_tree);

            self.sender
                .send(RendererMessage::RePaint(display_list))
                .expect("Can't send display list");
        }
    }

    fn load_html_local(&mut self, path: String) {
        if let Ok(html) = fs::read_to_string(&path) {
            let dom = parsing::parse_html(html);
            self.layout_engine.load_dom_tree(&dom);
            self.document = Some(dom);
        } else {
            self.sender
                .send(RendererMessage::ResourceNotFound(path))
                .expect("Can't send response");
        }
    }

    fn load_css_local(&mut self, path: String) {
        if let Ok(css) = fs::read_to_string(&path) {
            let stylesheet = parsing::parse_css(css);
            self.layout_engine.append_stylesheet(stylesheet);
        } else {
            self.sender
                .send(RendererMessage::ResourceNotFound(path))
                .expect("Can't send response");
        }
    }
}

fn init_logging(id: &str) {
    let mut log_dir = dirs::home_dir().expect("Home directory not found");
    log_dir.push("/tmp/moon");
    std::fs::create_dir_all(&log_dir).expect("Cannot create log directory");

    log_dir.push(format!("renderer_log.txt"));
    simple_logging::log_to_file(log_dir, log::LevelFilter::Debug).expect("Can not open log file");
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
                    break;
                }
                renderer.handle_kernel_msg(msg);
            }
            Err(_) => break,
        }
    }
}
