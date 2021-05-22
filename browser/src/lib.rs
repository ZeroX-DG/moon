mod window;
mod kernel;
mod renderer_client;
mod display_frame;
mod kernel_wrapper;
mod messenger;

use std::io::Read;
use kernel_wrapper::KernelWrapper;

fn read_file(path: String) -> String {
    let mut file = std::fs::File::open(path).expect("Unable to open file");
    let mut result = String::new();

    file.read_to_string(&mut result).expect("Unable to read file!");

    return result;
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn run_test(html: String, css: String, viewport: (u32, u32)) {
    let mut kernel_wrapper = KernelWrapper::new();
    kernel_wrapper.manual_load(read_file(html), read_file(css));
    kernel_wrapper.run_event_loop(viewport);
}

