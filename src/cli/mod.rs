mod action;

use clap::{App, Arg, ArgMatches};
pub use action::*;

const AUTHOR: &'static str = "Viet-Hung Nguyen <viethungax@gmail.com>";

pub fn accept_cli<'a>() -> ArgMatches<'a> {
    let html_file_arg = Arg::with_name("html")
        .long("html")
        .required(false)
        .takes_value(true);

    let css_file_arg = Arg::with_name("css")
        .long("css")
        .required(false)
        .takes_value(true);

    let size_arg = Arg::with_name("size")
        .long("size")
        .required(true)
        .takes_value(true);

    let ouput_arg = Arg::with_name("output")
        .long("output")
        .required(true)
        .takes_value(true);

    let render_testing_subcommand = App::new("render-testing")
        .about("Start a rendering process of Moon and render once")
        .version(rendering::version())
        .author(AUTHOR)
        .arg(html_file_arg.clone().required(true))
        .arg(css_file_arg.clone().required(true))
        .arg(size_arg.clone())
        .arg(ouput_arg.clone());

    let kernel_testing_subcommand = App::new("kernel-testing")
        .about("Start a kernel process of Moon and render once")
        .version(kernel::version())
        .author(AUTHOR)
        .arg(html_file_arg.clone().required(true))
        .arg(css_file_arg.clone().required(true))
        .arg(size_arg.clone());

    let rendering_subcommand = App::new("render")
        .about("Start a rendering process of Moon and connect to main process")
        .version(rendering::version())
        .author(AUTHOR);

    App::new("Moon Renderer")
        .version("1.0")
        .author(AUTHOR)
        .about("Moon web browser!")
        .subcommand(render_testing_subcommand)
        .subcommand(kernel_testing_subcommand)
        .subcommand(rendering_subcommand)
        .get_matches()
}

