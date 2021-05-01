use clap::ArgMatches;
use std::str::FromStr;

pub enum Action {
    RenderTesting(RenderTestingParams),
    Kernel
}

pub struct RenderTestingParams {
    pub html: String,
    pub css: String,
    pub size: (u32, u32),
    pub output: String
}

pub fn get_action<'a>(matches: ArgMatches<'a>) -> Action {
    if let Some(matches) = matches.subcommand_matches("render-testing") {
        let html: String = get_arg(&matches, "html").unwrap();
        let css: String = get_arg(&matches, "css").unwrap();
        let raw_size: String = get_arg(&matches, "size").unwrap();
        let output: String = get_arg(&matches, "output").unwrap();

        let size = match &raw_size.split('x')
            .filter_map(|size| size.parse::<u32>().ok())
            .take(2)
            .collect::<Vec<u32>>()[..] {
                &[width, height, ..] => (width, height),
                _ => unreachable!()
            };

        return Action::RenderTesting(RenderTestingParams {
            html,
            css,
            output,
            size
        });
    }

    Action::Kernel
}

fn get_arg<'a, T: FromStr>(matches: &ArgMatches, name: &'a str) -> Option<T> {
    matches.value_of(name)
        .map(|arg_str| arg_str.parse::<T>().ok())
        .unwrap_or(None)
}

