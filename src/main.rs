mod cli;

use image::{ImageBuffer, Rgba};
use render::page::Page;
use shared::primitive::Size;
use simplelog::*;
use std::io::Read;
use url::parser::URLParser;

fn read_file(path: String) -> String {
    let mut file = std::fs::File::open(path).expect("Unable to open file");
    let mut result = String::new();

    file.read_to_string(&mut result)
        .expect("Unable to read file!");

    return result;
}

fn main() {
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
            let html_code = read_file(params.html_path.clone());
            let viewport = params.viewport_size;
            let output_path = params.output_path;

            let absolute_html_path = std::fs::canonicalize(params.html_path).unwrap();
            let absolute_path = absolute_html_path.parent().unwrap();
            let absolute_path_url = format!("file://{}/", absolute_path.to_str().unwrap());
            let base_url = URLParser::parse(&absolute_path_url, None).unwrap();

            let (width, height) = viewport;

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut page = Page::new(Size::new(width as f32, height as f32)).await;
                page.load_html(html_code.to_string(), base_url).await;
                let bitmap = page.bitmap().unwrap().clone();

                let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, bitmap).unwrap();
                buffer.save(output_path).unwrap();
            });
        }
        cli::Action::StartMain => {
            main::start_main();
        }
    }
}
