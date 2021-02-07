use clap::{App, Arg};

pub enum Ops {
    LocalTest {
        html_path: String,
        css_path: String
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
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("css")
                .long("css")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    Ops::LocalTest {
        html_path: m.value_of("html").unwrap().to_string(),
        css_path: m.value_of("css").unwrap().to_string()
    }
}
