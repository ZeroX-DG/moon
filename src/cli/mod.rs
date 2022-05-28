mod action;

pub use action::*;
use clap::{App, Arg, ArgMatches};

const AUTHOR: &'static str = "Viet-Hung Nguyen <viethungax@gmail.com>";

pub fn accept_cli<'a>() -> ArgMatches<'a> {
    let html_file_arg = Arg::with_name("html")
        .long("html")
        .required(false)
        .takes_value(true);

    let size_arg = Arg::with_name("size")
        .long("size")
        .required(true)
        .takes_value(true);

    let once_flag = Arg::with_name("once").long("once");

    let ouput_arg = Arg::with_name("output")
        .long("output")
        .required(true)
        .takes_value(true);

    let render_once_subcommand = App::new("render")
        .about("Start a rendering process of Moon and render once")
        .author(AUTHOR)
        .arg(html_file_arg.clone().required(true))
        .arg(size_arg.clone())
        .arg(once_flag.clone())
        .arg(ouput_arg.clone());

    App::new("Moon Renderer")
        .author(AUTHOR)
        .about("Moon web browser!")
        .subcommand(render_once_subcommand)
        .get_matches()
}
