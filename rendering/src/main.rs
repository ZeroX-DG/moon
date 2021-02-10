mod cli;
mod layout_engine;
mod paint;
mod parsing;
mod renderer;

use cli::*;

use futures::executor::block_on;
use image::{ImageBuffer, Rgba};
use ipc::IpcRenderer;
use layout::box_model::Rect;
use message::{BrowserMessage, MessageToKernel};

use renderer::Renderer;

fn init_logging() {
    let mut log_dir = dirs::home_dir().expect("Home directory not found");
    log_dir.push("/tmp/moon");
    std::fs::create_dir_all(&log_dir).expect("Cannot create log directory");

    log_dir.push(format!("renderer_log.txt"));
    simple_logging::log_to_file(log_dir, log::LevelFilter::Debug).expect("Can not open log file");
}

fn save_frame_to_file(frame: &[u8], file: &str, viewport: &Rect) {
    let buffer =
        ImageBuffer::<Rgba<u8>, _>::from_raw(viewport.width as u32, viewport.height as u32, frame)
            .unwrap();

    buffer.save(file).unwrap();
}

fn perform_handshake(ipc: &IpcRenderer<BrowserMessage>, id: u16) -> Result<(), String> {
    ipc.sender
        .send(BrowserMessage::ToKernel(MessageToKernel::Syn(id)))
        .map_err(|e| e.to_string())?;
    log::info!("SYN sent!");

    loop {
        match ipc.receiver.recv().map_err(|e| e.to_string())? {
            BrowserMessage::ToRenderer(message::MessageToRenderer::SynAck(id)) => {
                log::info!("SYN-ACK received!");
                ipc.sender
                    .send(BrowserMessage::ToKernel(MessageToKernel::Ack(id)))
                    .map_err(|e| e.to_string())?;
                log::info!("ACK sent!");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn run_headless_mode(
    mut renderer: Renderer,
    html_path: String,
    css_path: String,
    output_path: String,
) {
    let mut html_file = match std::fs::File::open(&html_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!(
                "Unable to open HTML file({:?}): {:?}",
                html_path,
                e.to_string()
            );
            eprintln!(
                "Unable to open HTML file({:?}): {:?}",
                html_path,
                e.to_string()
            );
            return;
        }
    };
    let mut css_file = match std::fs::File::open(&css_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!(
                "Unable to open CSS file({:?}): {:?}",
                css_path,
                e.to_string()
            );
            eprintln!(
                "Unable to open CSS file({:?}): {:?}",
                css_path,
                e.to_string()
            );
            return;
        }
    };

    renderer.load_html(&mut html_file);
    renderer.load_css(&mut css_file);

    if let Some(frame) = renderer.repaint().await {
        save_frame_to_file(&frame, &output_path, renderer.viewport());
    }
}

async fn run_nonheadless_mode(mut renderer: Renderer) {
    let ipc = IpcRenderer::<BrowserMessage>::new();

    perform_handshake(&ipc, renderer.id()).expect("Unable to perform handshake with server");

    loop {
        match ipc.receiver.recv() {
            Ok(BrowserMessage::ToRenderer(msg)) => renderer.handle_msg(msg),
            Ok(msg) => unreachable!("Unrecognized message: {:#?}", msg),
            Err(e) => {
                log::error!("Error while receiving msg: {:#?}", e);
            }
        }

        if let Some(frame) = renderer.repaint().await {
            ipc.sender
                .send(BrowserMessage::ToKernel(MessageToKernel::RePaint(frame)))
                .unwrap();
        }
    }
}

async fn run() {
    init_logging();
    let ops = accept_cli();

    let viewport = Rect {
        x: 0.,
        y: 0.,
        width: 500.,
        height: 300.,
    };

    match ops {
        Ops::Headless {
            html_path,
            css_path,
            output_path,
        } => {
            let renderer = Renderer::new(0, viewport).await;
            run_headless_mode(renderer, html_path, css_path, output_path).await
        }
        Ops::NonHeadless { id } => {
            let renderer = Renderer::new(id, viewport).await;
            run_nonheadless_mode(renderer).await;
        }
    };
}

fn main() {
    block_on(run());
}
