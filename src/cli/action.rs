use clap::ArgMatches;
use std::str::FromStr;

pub enum Action {
    RenderOnce(RenderOnceParams),
}

pub struct RenderOnceParams {
    pub html_path: String,
    pub css_path: String,
    pub viewport_size: (u32, u32),
    pub output_path: String,
}

pub fn get_action<'a>(matches: ArgMatches<'a>) -> Action {
    if let Some(matches) = matches.subcommand_matches("render") {
        let html: String = get_arg(&matches, "html").unwrap();
        let css: String = get_arg(&matches, "css").unwrap();
        let raw_size: String = get_arg(&matches, "size").unwrap();
        let output_path: String = get_arg(&matches, "output").unwrap();

        let is_render_once = get_flag(&matches, "once");

        let viewport_size = parse_size(&raw_size);

        if is_render_once {
            return Action::RenderOnce(RenderOnceParams {
                html_path: html,
                css_path: css,
                output_path,
                viewport_size,
            });
        }
    }

    unreachable!("Invalid action provided!");
}

fn parse_size(raw_size: &str) -> (u32, u32) {
    let size_params = raw_size
        .split('x')
        .filter_map(|size| size.parse::<u32>().ok())
        .take(2)
        .collect::<Vec<u32>>();

    match &size_params[..] {
        &[width, height, ..] => (width, height),
        _ => unreachable!(),
    }
}

fn get_arg<'a, T: FromStr>(matches: &ArgMatches, name: &'a str) -> Option<T> {
    matches
        .value_of(name)
        .map(|arg_str| arg_str.parse::<T>().ok())
        .unwrap_or(None)
}

fn get_flag<'a>(matches: &ArgMatches, flag: &'a str) -> bool {
    matches.is_present(flag)
}
