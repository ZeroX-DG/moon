mod cli;

use std::io::Read;
use futures::executor::block_on;
use image::{ImageBuffer, Rgba};

fn main() {
    block_on(async_main());
}

async fn async_main() {
    let action = cli::get_action(cli::accept_cli());

    match action {
        cli::Action::RenderTesting(params) => {
            let mut html_file = std::fs::File::open(&params.html).unwrap();
            let mut css_file = std::fs::File::open(&params.css).unwrap();

            let mut html_code = String::new();
            let mut css_code = String::new();

            html_file.read_to_string(&mut html_code).expect("Unable to read HTML file");
            css_file.read_to_string(&mut css_code).expect("Unable to read CSS file");

            if let Some(bitmap) = rendering::render_once(html_code, css_code, params.size).await {
                let buffer =
                    ImageBuffer::<Rgba<u8>, _>::from_raw(params.size.0, params.size.1, bitmap)
                        .unwrap();
                buffer.save(params.output).unwrap();
            }
        }
        _ => {}
    }
}

