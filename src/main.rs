mod cli;

use image::{ImageBuffer, Rgba};
use simplelog::*;
use std::io::Read;

fn read_file(path: String) -> String {
    let mut file = std::fs::File::open(path).expect("Unable to open file");
    let mut result = String::new();

    file.read_to_string(&mut result)
        .expect("Unable to read file!");

    return result;
}

#[tokio::main]
async fn main() {
    let config = ConfigBuilder::new()
        .add_filter_ignore_str("wgpu")
        .add_filter_ignore_str("gfx_backend_vulkan")
        .add_filter_ignore_str("naga")
        .set_target_level(LevelFilter::Info)
        .build();
    TermLogger::init(
        LevelFilter::Debug,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let action = cli::get_action(cli::accept_cli());

    match action {
        cli::Action::RenderOnce(params) => {
            let html_code = read_file(params.html_path);
            let viewport = params.viewport_size;
            let output_path = params.output_path;

            let bitmap = render::render_once(html_code.to_string(), viewport).await;

            let (width, height) = viewport;

            let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, bitmap).unwrap();
            buffer.save(output_path).unwrap();
        }
    }

    //     let html_code = include_str!("../fixtures/test_text.html");
    //     let viewport = (500, 300);
    //     let output_path = "image.png";
    //
    //     let bitmap = render::render_once(html_code.to_string(), viewport).await;
    //
    //     let (width, height) = viewport;
    //
    //     let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, bitmap).unwrap();
    //     buffer.save(output_path).unwrap();
}
