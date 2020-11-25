mod parsing;
mod layouting;

use dom::dom_ref::NodeRef;
use css::cssom::stylesheet::StyleSheet;

use ipc::{Client, Sender};
use std::io::{self, Stdin, StdinLock, Stdout, StdoutLock};
use std::fs;
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
    sender: &'a mut Sender<RendererMessage>,
    document: Option<NodeRef>,
    stylesheets: Vec<StyleSheet>,
}

impl<'a> Renderer<'a> {
    pub fn new(sender: &'a mut Sender<RendererMessage>) -> Self {
        Self {
            id: nanoid::simple(),
            sender,
            document: None,
            stylesheets: Vec::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn handle_kernel_msg(&mut self, msg: KernelMessage) {
        match msg {
            KernelMessage::LoadHTMLLocal(path) => {
                self.load_html_local(path);
                self.reflow(self.document.clone().unwrap());
            },
            KernelMessage::LoadCSSLocal(path) => {
                self.load_css_local(path);
                self.reflow(self.document.clone().unwrap());
            },
            _ => {
                log::debug!("Unknown kernel message: {:?}", msg);
            }
        }
    }

    fn reflow(&mut self, root: NodeRef) {
        let new_layout = layouting::layout(
            &root,
            &self.stylesheets,
            500.0,
            300.0
        );

        let display_list = painting::build_display_list(&new_layout);

        self.sender.send(RendererMessage::RePaint(display_list))
            .expect("Can't send frame");
    }

    fn load_html_local(&mut self, path: String) {
        let html = fs::read_to_string(path).expect("Unable to read HTML file");
        let dom = parsing::parse_html(html);
        self.document = Some(dom);
    }

    fn load_css_local(&mut self, path: String) {
        let css = fs::read_to_string(path).expect("Unable to read CSS file");
        let stylesheet = parsing::parse_css(css);
        self.stylesheets.push(stylesheet);
    }
}

fn init_logging(id: &str) {
    let mut log_dir = dirs::home_dir().expect("Home directory not found");
    log_dir.push("/tmp/moon");
    std::fs::create_dir_all(&log_dir).expect("Cannot create log directory");

    log_dir.push(format!("renderer_log.txt"));
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
}

