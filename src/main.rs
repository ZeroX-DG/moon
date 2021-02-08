mod cli;
mod kernel;
mod renderer;
mod window;

use flume::{Receiver, Sender};
use kernel::Kernel;
// use logging::init_logging;
use ipc::IpcMain;
use message::BrowserMessage;
use std::{
    sync::{Arc, Mutex},
    thread,
};

const IPC_PORT: u16 = 4444;

pub enum UIAction {
    RePaint(Vec<u8>),
}

pub enum KernelAction {
    NewTabTest,
}

/// Kernel thread
/// This thread handles these tasks:
/// - Start IPC main
/// - Constantly receive message from renderers
/// - Constantly execute command according to message
fn run_kernel_thread(tx_ui: Sender<UIAction>, rx_kernel: Receiver<KernelAction>) {
    thread::spawn(move || {
        let kernel = Arc::new(Mutex::new(Kernel::new()));
        let kernel_clone = kernel.clone();

        // Start IPC main
        let mut ipc = IpcMain::<BrowserMessage>::new();
        ipc.run(IPC_PORT);

        thread::spawn(move || loop {
            match rx_kernel.recv() {
                Ok(action) => match action {
                    KernelAction::NewTabTest => {
                        kernel_clone.lock().unwrap().new_tab();
                    }
                },
                _ => {}
            }
        });

        loop {
            match ipc.receive() {
                Ok((reply, msg)) => match msg {
                    BrowserMessage::ToKernel(msg) => {
                        kernel.lock().unwrap().handle_msg(reply, msg, &ipc)
                    }
                    _ => unreachable!("Unknown msg: {:#?}", msg),
                },

                // we have no client/tab yet, sleep between loop so we dont
                // make the CPU scream
                Err(ipc::IpcMainReceiveError::NoConnections) => {
                    thread::sleep(std::time::Duration::from_millis(10));
                }

                Err(ipc::IpcMainReceiveError::Other(e)) => {
                    eprintln!("Error while receiving msg: {:#?}", e);
                }
            }
        }
    });
}

/// UI Thread (Main thread)
/// This thread handles these tasks:
/// - Run main UI loop
/// - Receive pixel buffer & render to screen
fn run_ui_thread(tx_kernel: Sender<KernelAction>, rx_ui: Receiver<UIAction>) {
    // window::run_ui_loop();
    tx_kernel.send(KernelAction::NewTabTest).unwrap();
    window::run_ui_loop();
}

fn main() {
    // init_logging();
    let matches = cli::accept_cli();

    // Communication channel between Kernel & UI thread
    let (tx_kernel, rx_kernel) = flume::bounded(1);
    let (tx_ui, rx_ui) = flume::bounded(1);

    run_kernel_thread(tx_ui, rx_kernel);
    run_ui_thread(tx_kernel, rx_ui);
}
