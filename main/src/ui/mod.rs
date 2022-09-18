mod content_area;
mod primary_bar;

use gtk::gdk::EventMask;
use gtk::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::glib::Bytes;
use gtk::{prelude::*, Orientation};
use gtk::{Application, ApplicationWindow};

use self::content_area::ContentArea;
use self::primary_bar::PrimaryBar;

pub struct UI {
    pub app: Application,
    pub window: ApplicationWindow,
    pub content_area: ContentArea,
    pub primary_bar: PrimaryBar,

    pub spinner: gtk::Image,
}

impl UI {
    pub fn new(app: Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(&app)
            .title("Moon")
            .default_width(1200)
            .default_height(600)
            .events(EventMask::BUTTON_PRESS_MASK)
            .build();

        let container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        let primary_bar = PrimaryBar::new(&container);
        let content_area = ContentArea::new(&container);

        let header_bar = gtk::HeaderBar::builder().show_close_button(true).build();

        let spinner = gtk::Image::from_file("resources/loading.gif");
        header_bar.add(&spinner);

        window.add(&container);
        window.set_titlebar(Some(&header_bar));

        Self {
            app,
            window,
            content_area,
            primary_bar,
            spinner,
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    pub fn set_url(&mut self, url: &str) {
        self.primary_bar.url_entry.set_text(url);
    }

    pub fn set_loading_start(&mut self) {
        self.spinner.show();
    }

    pub fn set_loading_finished(&mut self) {
        self.spinner.hide();
    }

    pub fn set_web_content_bitmap(&mut self, bitmap: Vec<u8>) {
        let (width, height) = self.content_area.render_area_size();

        if (width * height * 4) as usize > bitmap.len() {
            return;
        }

        let bytes = Bytes::from_owned(bitmap);
        let pixbuf = Pixbuf::from_bytes(&bytes, Colorspace::Rgb, true, 8, width, height, width * 4);
        self.content_area.set_content_pixbuf(pixbuf);
    }
}
