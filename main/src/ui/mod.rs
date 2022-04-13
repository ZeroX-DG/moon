mod primary_bar;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk_pixbuf::Pixbuf;
use gtk::{prelude::*, DrawingArea, Orientation};
use gtk::{Application, ApplicationWindow};

use self::primary_bar::PrimaryBar;

pub struct UI {
    pub app: Application,
    pub window: ApplicationWindow,
    pub content_area: DrawingArea,
    pub primary_bar: PrimaryBar,
    web_content: Rc<RefCell<Option<Pixbuf>>>,
}

impl UI {
    pub fn new(app: Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(&app)
            .title("Moon")
            .default_width(1200)
            .default_height(600)
            .build();

        let content_area = DrawingArea::builder().hexpand(true).vexpand(true).build();

        let web_content: Rc<RefCell<Option<Pixbuf>>> = Rc::new(RefCell::new(None));

        let web_content_clone = web_content.clone();

        content_area.connect_draw(move |_, context| {
            if let Some(pixbuf) = &*web_content_clone.borrow() {
                context.set_source_pixbuf(pixbuf, 0., 0.);
                context.paint().unwrap();
                return Inhibit(false);
            }

            Inhibit(true)
        });

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
            web_content,
        }
    }

    pub fn set_content_pixbuf(&mut self, content: Pixbuf) {
        self.web_content.borrow_mut().replace(content);
        self.content_area.queue_draw();
    }
}
