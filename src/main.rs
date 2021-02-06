mod kernel;
// mod logging;
mod renderer;
mod window;

use clap::{App, Arg, ArgMatches};
use kernel::Kernel;
// use logging::init_logging;
use message::{BrowserMessage, MessageToRenderer};
use std::sync::{Arc, Mutex};

fn init_cli<'a>() -> ArgMatches<'a> {
    App::new("Moon")
        .version("1.0")
        .author("Viet-Hung Nguyen <viethungax@gmail.com>")
        .about("A rusty web browser")
        .arg(
            Arg::with_name("html")
                .required(true)
                .long("html")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("css")
                .required(true)
                .long("css")
                .takes_value(true),
        )
        .get_matches()
}

fn main() {
    // init_logging();
    let matches = init_cli();

    let kernel = Arc::new(Mutex::new(Kernel::new()));

    let kernel_clone = kernel.clone();

    // Initialize a channel to pass the bitmap data
    // back to the UI loop for rendering.
    let (tx, rx) = flume::bounded::<Vec<u8>>(1);

    std::thread::spawn(move || {
        kernel_clone.lock().unwrap().run(tx);
    });

    window::run_ui_loop(rx);

    let mut kernel = kernel.lock().unwrap();
    let renderer_id = kernel.spawn_new_renderer();

    let renderer = kernel.get_renderer(renderer_id);

    if let Some(html_path) = matches.value_of("html") {
        renderer.send(BrowserMessage::ToRenderer(
            MessageToRenderer::LoadHTMLLocal(html_path.to_string()),
        ));
    }

    if let Some(css_path) = matches.value_of("css") {
        renderer.send(BrowserMessage::ToRenderer(MessageToRenderer::LoadCSSLocal(
            css_path.to_string(),
        )));
    }
}
