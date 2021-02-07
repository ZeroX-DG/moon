mod cli;
mod layout_engine;
mod paint;
mod parsing;

use cli::*;
use dom::dom_ref::NodeRef;
use futures::executor::block_on;
use image::{ImageBuffer, Rgba};
use ipc::IpcRenderer;
use layout::box_model::Rect;
use layout_engine::LayoutEngine;
use message::{BrowserMessage, MessageToKernel};
use paint::Painter;
use parsing::{parse_css, parse_html};
use std::io::Read;

pub struct Renderer {
    id: u16,
    document: Option<NodeRef>,
    layout_engine: LayoutEngine,
    painter: Painter,
    viewport: Rect,
}

impl Renderer {
    pub async fn new(id: u16, viewport: Rect) -> Self {
        let painter = Painter::new(viewport.width as u32, viewport.height as u32).await;
        Self {
            id,
            document: None,
            layout_engine: LayoutEngine::new(viewport.clone()),
            painter,
            viewport,
        }
    }

    pub fn id(&self) -> &u16 {
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

    pub async fn repaint(&mut self) -> Option<Vec<u8>> {
        if let Some(layout_tree) = self.layout_engine.layout_tree() {
            let display_list = painting::build_display_list(layout_tree);
            painting::paint(&display_list, &mut self.painter);

            self.painter.paint().await
        } else {
            None
        }
    }
}

fn init_logging(_id: &str) {
    let mut log_dir = dirs::home_dir().expect("Home directory not found");
    log_dir.push("/tmp/moon");
    std::fs::create_dir_all(&log_dir).expect("Cannot create log directory");

    log_dir.push(format!("renderer_log.txt"));
    simple_logging::log_to_file(log_dir, log::LevelFilter::Debug).expect("Can not open log file");
}

fn save_frame_to_file(frame: &[u8], file: &str, viewport: &Rect) {
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
        viewport.width as u32,
        viewport.height as u32,
        frame,
    )
    .unwrap();

    buffer.save(file).unwrap();
}

fn perform_handshake(ipc: &IpcRenderer<BrowserMessage>, id: u16) -> Result<(), String> {
    ipc.sender.send(BrowserMessage::ToKernel(MessageToKernel::Syn(id)))
        .map_err(|e| e.to_string())?;

    loop {
        match ipc.receiver.recv().map_err(|e| e.to_string())? {
            BrowserMessage::ToRenderer(message::MessageToRenderer::SynAck(id)) => {
                ipc.sender.send(BrowserMessage::ToKernel(MessageToKernel::Ack(id)))
                    .map_err(|e| e.to_string())?;
                break;
            },
            _ => {}
        }
    }

    Ok(())
}

async fn run() {
    let ops = accept_cli();

    let viewport = Rect {
        x: 0.,
        y: 0.,
        width: 500.,
        height: 300.,
    };

    let mut renderer = Renderer::new(0, viewport.clone()).await;

    match ops {
        Ops::LocalTest { html_path, css_path, output_path } => {
            let mut html_file = std::fs::File::open(html_path).expect("Unable to open HTML file");
            let mut css_file = std::fs::File::open(css_path).expect("Unable to open CSS file");

            renderer.load_html(&mut html_file);
            renderer.load_css(&mut css_file);
            
            if let Some(frame) = renderer.repaint().await {
                save_frame_to_file(&frame, &output_path, &viewport);
            }
        }
        Ops::KernelConnect => {
            let ipc = IpcRenderer::<BrowserMessage>::new(4444);

            perform_handshake(&ipc, *renderer.id()).expect("Unable to perform handshake with server");

            if let Some(frame) = renderer.repaint().await {
                ipc.sender
                    .send(BrowserMessage::ToKernel(MessageToKernel::RePaint(frame)))
                    .unwrap();
            }
        }
    }
}

fn main() {
    block_on(run());
}
