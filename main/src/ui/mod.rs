mod primary_bar;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gtk::gdk_pixbuf::Pixbuf;
use gtk::{prelude::*, DrawingArea, Orientation};
use gtk::{Application, ApplicationWindow};
use shared::primitive::Size;

use crate::app::get_app_runtime;
use crate::delayed_task::DelayedTask;

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

        let debouncer: Arc<Mutex<Option<DelayedTask>>> = Arc::new(Mutex::new(None));

        content_area.connect_size_allocate(move |_, _| {
            if let Some(task) = &*debouncer.lock().unwrap() {
                task.clear();
            }
            debouncer
                .lock()
                .unwrap()
                .replace(DelayedTask::new(Duration::from_millis(200), || {
                    get_app_runtime().update_state(|state| {
                        let width = state.ui.content_area.allocated_width();
                        let height = state.ui.content_area.allocated_width();
                        let new_size = Size::new(width as f32, height as f32);
                        state.active_tab_mut().resize(new_size);
                    });
                }));
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

    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    pub fn set_content_pixbuf(&mut self, content: Pixbuf) {
        self.web_content.borrow_mut().replace(content);
        self.content_area.queue_draw();
    }
}
