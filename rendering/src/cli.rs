use clap::{App, Arg};

pub enum Ops {
    Headless {
        html_path: String,
        css_path: String,
        output_path: String
    },
    NonHeadless {
        id: u16
    }
}

pub fn accept_cli() -> Ops {
    let m = App::new("Moon Renderer")
        .version("1.0")
        .author("Viet-Hung Nguyen <viethungax@gmail.com>")
        .about("Renderer for moon browser")
        .arg(
            Arg::with_name("html")
                .long("html")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("css")
                .long("css")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("mode")
                .long("mode")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .takes_value(true),
        )
        .get_matches();

    match m.value_of("mode") {
        Some("headless") if m.is_present("html") && m.is_present("css") && m.is_present("output") => {
            Ops::Headless {
                html_path: m.value_of("html").unwrap().to_string(),
                css_path: m.value_of("css").unwrap().to_string(),
                output_path: m.value_of("output").unwrap().to_string()
            }
        },
        _ if m.is_present("id") => Ops::NonHeadless {
            id: m.value_of("id").unwrap().to_string().parse::<u16>().unwrap()
        },
        _ => panic!("Invalid CLI options")
    }
}
