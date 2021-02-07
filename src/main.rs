mod kernel;
mod cli;
mod renderer;
mod window;

use kernel::Kernel;
// use logging::init_logging;
use message::{BrowserMessage};
use std::thread;
use ipc::IpcMain;

const IPC_PORT: u16 = 4444;

/// IPC thread
/// This thread handles these tasks:
/// - Start IPC main
/// - Constantly receive message from renderers
/// - Constantly execute command according to message
fn run_ipc_thread() {
    thread::spawn(|| {
        let mut kernel = Kernel::new();

        // Start IPC main
        let mut ipc = IpcMain::<BrowserMessage>::new();
        ipc.run(IPC_PORT);

        loop {
            match ipc.receive() {
                Ok((reply, msg)) => match msg {
                    BrowserMessage::ToKernel(msg) => kernel.handle_msg(reply, msg, &ipc),
                    _ => unreachable!("Unknown msg: {:#?}", msg)
                }

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
fn run_ui_thread() {
    // window::run_ui_loop();
}

fn main() {
    // init_logging();
    let matches = cli::accept_cli();

    // Communication channel between IPC & UI thread
    // let (tx_buffer, rx_buffer) = flume::bounded(1);

    run_ipc_thread();
    run_ui_thread();
}
