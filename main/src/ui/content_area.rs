use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

use gtk::{
    gdk::EventMask,
    gdk_pixbuf::Pixbuf,
    prelude::{GdkContextExt, GtkMenuExtManual},
    traits::{ContainerExt, GtkMenuItemExt, MenuShellExt, WidgetExt},
    DrawingArea, Inhibit,
};
use shared::primitive::Size;

use crate::{app::get_app_runtime, delayed_task::DelayedTask};

pub struct ContentArea {
    render_area: DrawingArea,
    web_content_pixbuf: Rc<RefCell<Option<Pixbuf>>>,
}

impl ContentArea {
    pub fn new(container: &gtk::Box) -> Self {
        let render_area = DrawingArea::builder()
            .hexpand(true)
            .vexpand(true)
            .events(EventMask::BUTTON_PRESS_MASK | EventMask::SMOOTH_SCROLL_MASK)
            .build();

        let web_content_pixbuf: Rc<RefCell<Option<Pixbuf>>> = Rc::new(RefCell::new(None));

        let web_content_clone = web_content_pixbuf.clone();

        render_area.connect_draw(move |_, context| {
            if let Some(pixbuf) = &*web_content_clone.borrow() {
                context.set_source_pixbuf(pixbuf, 0., 0.);
                context.paint().unwrap();
                return Inhibit(false);
            }

            Inhibit(true)
        });

        let debouncer: Arc<Mutex<Option<DelayedTask>>> = Arc::new(Mutex::new(None));

        render_area.connect_size_allocate(move |_, _| {
            if let Some(task) = &*debouncer.lock().unwrap() {
                task.clear();
            }
            debouncer
                .lock()
                .unwrap()
                .replace(DelayedTask::new(Duration::from_millis(200), || {
                    get_app_runtime().update_state(|state| {
                        let (width, height) = state.ui.content_area.render_area_size();
                        let new_size = Size::new(width as f32, height as f32);
                        state.browser().resize(new_size);
                    });
                }));
        });

        render_area.connect_scroll_event(move |_, event| {
            let delta_y = event.delta().1 as f32;
            get_app_runtime().update_state(move |state| {
                state.browser().scroll(delta_y * 1.2);
            });

            Inhibit(false)
        });

        render_area.connect_button_press_event(|_, event| {
            let right_button = 3;
            if event.button() == right_button {
                let menu = gtk::Menu::new();
                let item = gtk::MenuItem::with_label("View Source");

                item.connect_activate(|_| {
                    get_app_runtime().update_state(|state| {
                        state.browser().view_source_current_tab();
                    });
                });

                menu.append(&item);

                menu.show_all();
                menu.popup_easy(event.button(), event.time());
            }
            Inhibit(true)
        });

        container.add(&render_area);

        Self {
            render_area,
            web_content_pixbuf,
        }
    }

    pub fn render_area_size(&self) -> (i32, i32) {
        let width = self.render_area.allocated_width();
        let height = self.render_area.allocated_height();
        (width, height)
    }

    pub fn set_content_pixbuf(&mut self, content: Pixbuf) {
        self.web_content_pixbuf.borrow_mut().replace(content);
        self.render_area.queue_draw();
    }
}
