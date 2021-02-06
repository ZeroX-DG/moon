use clap::{App, Arg};

pub struct Ops {
    pub action: Action,
    pub output: Output
}

pub enum Action {
    SimpleTest {
        html_path: String,
        css_path: String
    }
}

pub enum Output {
    File(String),
    Kernel
}

pub fn accept_cli() -> Option<Ops> {
    let m = App::new("Moon Renderer")
        .version("1.0")
        .author("Viet-Hung Nguyen <viethungax@gmail.com>")
        .about("Renderer for moon browser")
        .arg(
            Arg::with_name("html")
                .long("html")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("css")
                .long("css")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .required(false)
                .takes_value(true)
        )
        .get_matches();

    let output = match m.value_of("output") {
        Some(file) => Output::File(file.to_string()),
        None => Output::Kernel
    };
    
    if m.is_present("html") && m.is_present("css") {
        return Some(Ops {
            action: Action::SimpleTest {
                html_path: m.value_of("html").unwrap().to_string(),
                css_path: m.value_of("css").unwrap().to_string()
            },
            output
        })
    }

    None
}