mod cli;

use std::io::Read;
use futures::executor::block_on;
use image::{ImageBuffer, Rgba};
use ipc::IpcRenderer;

fn main() {
    block_on(async_main());
}

fn read_file(path: String) -> String {
    let mut file = std::fs::File::open(path).expect("Unable to open file");
    let mut result = String::new();

    file.read_to_string(&mut result).expect("Unable to read file!");

    return result;
}

async fn async_main() {
    let action = cli::get_action(cli::accept_cli());

    match action {
        cli::Action::RenderTesting(params) => {
            let html_code = read_file(params.html);
            let css_code = read_file(params.css);

            if let Some(bitmap) = rendering::render_once(html_code, css_code, params.size).await {
                let buffer =
                    ImageBuffer::<Rgba<u8>, _>::from_raw(params.size.0, params.size.1, bitmap)
                        .unwrap();
                buffer.save(params.output).unwrap();
            }
        }
        cli::Action::KernelTesting(params) => {
            unimplemented!()
        }
        cli::Action::Rendering => {
        }
    }
}

