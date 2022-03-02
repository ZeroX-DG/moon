mod primary_bar;

use gtk::{prelude::*, Orientation};
use gtk::{Application, ApplicationWindow, Image};

use self::primary_bar::PrimaryBar;

pub struct UI {
    pub app: Application,
    pub window: ApplicationWindow,
    pub content_area: Image,
    pub primary_bar: PrimaryBar,
}

impl UI {
    pub fn new(app: Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(&app)
            .title("Moon")
            .default_width(1200)
            .default_height(600)
            .build();

        let content_area = Image::builder().hexpand(true).vexpand(true).build();

        let container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        let primary_bar = PrimaryBar::new(&container);

        container.add(&content_area);
        window.add(&container);

        Self {
            app,
            window,
            content_area,
            primary_bar,
        }
    }
}
