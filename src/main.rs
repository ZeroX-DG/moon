mod cli;
mod kernel;
mod renderer;
mod window;
mod logging;

use flume::{Receiver, Sender};
use kernel::Kernel;
use logging::init_logging;
use ipc::IpcMain;
use message::{BrowserMessage, MessageToRenderer};
use std::{
    sync::{Arc, Mutex},
    thread,
};

const IPC_PORT: u16 = 4444;

pub enum UIAction {
    RePaint(Vec<u8>),
}

pub enum KernelAction {
    NewTabTest {
        html_file: String,
        css_file: String
    },
    CleanUp
}

/// Kernel thread
/// This thread handles these tasks:
/// - Start IPC main
/// - Constantly receive message from renderers
/// - Constantly execute command according to message
fn run_kernel_thread(tx_ui: Sender<UIAction>, rx_kernel: Receiver<KernelAction>) {
    let kernel = Arc::new(Mutex::new(Kernel::new()));
    let kernel_clone = kernel.clone();

    // Start IPC main
    let mut ipc = IpcMain::<BrowserMessage>::new();
    ipc.run(IPC_PORT);

    thread::spawn(move || loop {
        match rx_kernel.recv() {
            Ok(action) => match action {
                KernelAction::NewTabTest { html_file, css_file } => {
                    let id = {
                        let mut kernel = kernel_clone.lock().unwrap();
                        let id = kernel.new_tab();

                        id
                    };

                    while !kernel_clone.lock().unwrap().get_renderer(id).is_ready() {
                        thread::sleep(std::time::Duration::from_millis(1000));
                    }

                    let kernel = kernel_clone.lock().unwrap();
                    let tab = kernel.get_renderer(id);

                    tab.send(BrowserMessage::ToRenderer(MessageToRenderer::LoadHTMLLocal(html_file)));
                    tab.send(BrowserMessage::ToRenderer(MessageToRenderer::LoadCSSLocal(css_file)));
                }
                KernelAction::CleanUp => {
                    kernel_clone.lock().unwrap().clean_up();
                }
            },
            _ => {}
        }
    });

    thread::spawn(move || loop {
        match ipc.receive() {
            Ok((reply, msg)) => match msg {
                BrowserMessage::ToKernel(msg) => {
                    kernel.lock().unwrap().handle_msg(reply, msg, &ipc, tx_ui.clone())
                }
                _ => unreachable!("Unknown msg: {:#?}", msg),
            },

            // we have no client/tab yet, sleep between loop so we dont
            // make the CPU scream
            Err(ipc::IpcMainReceiveError::NoConnections) => {
                thread::sleep(std::time::Duration::from_millis(10));
            }

            Err(ipc::IpcMainReceiveError::Other(e)) => {
                log::error!("Error while receiving msg: {:#?}", e);
            }
        }
    });
}

/// UI Thread (Main thread)
/// This thread handles these tasks:
/// - Run main UI loop
/// - Receive pixel buffer & render to screen
fn run_ui_thread(tx_kernel: Sender<KernelAction>, rx_ui: Receiver<UIAction>) {
    window::run_ui_loop(tx_kernel, rx_ui);
}

fn main() {
    init_logging();
    let ops = cli::accept_cli();

    // Communication channel between Kernel & UI thread
    let (tx_kernel, rx_kernel) = flume::bounded(1);
    let (tx_ui, rx_ui) = flume::bounded(1);

    run_kernel_thread(tx_ui, rx_kernel);

    match ops {
        cli::Ops::LocalTest { html_path, css_path } => {
            tx_kernel.send(KernelAction::NewTabTest {
                html_file: html_path,
                css_file: css_path
            }).unwrap();
        }
    }

    run_ui_thread(tx_kernel, rx_ui);
}
