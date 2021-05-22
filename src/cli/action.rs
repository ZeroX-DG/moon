use clap::ArgMatches;
use std::str::FromStr;

pub enum Action {
    RenderTesting(RenderTestingParams),
    KernelTesting(KernelTestingParams),
    Rendering(RenderingParams)
}

pub struct RenderTestingParams {
    pub html: String,
    pub css: String,
    pub size: (u32, u32),
    pub output: String
}

pub struct KernelTestingParams {
    pub html: String,
    pub css: String,
    pub size: (u32, u32),
}

pub struct RenderingParams {
    pub id: String,
}

pub fn get_action<'a>(matches: ArgMatches<'a>) -> Action {
    if let Some(matches) = matches.subcommand_matches("render-testing") {
        let html: String = get_arg(&matches, "html").unwrap();
        let css: String = get_arg(&matches, "css").unwrap();
        let raw_size: String = get_arg(&matches, "size").unwrap();
        let output: String = get_arg(&matches, "output").unwrap();

        let size = translate_size(&raw_size);

        return Action::RenderTesting(RenderTestingParams {
            html,
            css,
            output,
            size
        });
    }

    if let Some(matches) = matches.subcommand_matches("kernel-testing") {
        let html: String = get_arg(&matches, "html").unwrap();
        let css: String = get_arg(&matches, "css").unwrap();
        let raw_size: String = get_arg(&matches, "size").unwrap();
        let size = translate_size(&raw_size);

        return Action::KernelTesting(KernelTestingParams {
            html,
            css,
            size
        });
    }

    if matches.subcommand_matches("render").is_some() {
        let renderer_id: String = get_arg(&matches, "id").unwrap();
        return Action::Rendering(RenderingParams {
            id: renderer_id
        });
    }
    
    unreachable!("Invalid action provided!");
}

fn translate_size(raw_size: &str) -> (u32, u32) {
    let size_params = raw_size.split('x')
        .filter_map(|size| size.parse::<u32>().ok())
        .take(2)
        .collect::<Vec<u32>>();

    match &size_params[..] {
        &[width, height, ..] => (width, height),
        _ => unreachable!()
    }
}

fn get_arg<'a, T: FromStr>(matches: &ArgMatches, name: &'a str) -> Option<T> {
    matches.value_of(name)
        .map(|arg_str| arg_str.parse::<T>().ok())
        .unwrap_or(None)
}

