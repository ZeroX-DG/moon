// mod kernel;
// mod logging;
// mod renderer_handler;
// mod window;

// use clap::{App, Arg, ArgMatches};
// use kernel::Kernel;
// use logging::init_logging;
// use message::KernelMessage;

// fn init_cli<'a>() -> ArgMatches<'a> {
//     App::new("Moon")
//         .version("1.0")
//         .author("Viet-Hung Nguyen <viethungax@gmail.com>")
//         .about("A rusty web browser")
//         .arg(
//             Arg::with_name("html")
//                 .required(true)
//                 .long("html")
//                 .takes_value(true),
//         )
//         .arg(
//             Arg::with_name("css")
//                 .required(true)
//                 .long("css")
//                 .takes_value(true),
//         )
//         .get_matches()
// }

fn main() {
    // init_logging();
    // let matches = init_cli();

    // let mut kernel = Kernel::new();

    // let renderer = kernel.renderer_handlers().new_renderer();

    // if let Some(html_path) = matches.value_of("html") {
    //     renderer
    //         .send(KernelMessage::LoadHTMLLocal(html_path.to_string()))
    //         .expect("Unable to send HTML path to renderer");
    // }

    // if let Some(css_path) = matches.value_of("css") {
    //     renderer
    //         .send(KernelMessage::LoadCSSLocal(css_path.to_string()))
    //         .expect("Unable to send CSS path to renderer");
    // }

    // // Initialize a channel to pass the bitmap data
    // // back to the UI loop for rendering.
    // let (tx, rx) = flume::bounded::<Vec<u8>>(1);

    // std::thread::spawn(move || {
    //     kernel.main_loop(tx);
    // });

    // window::run_ui_loop(rx);
}
