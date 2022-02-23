use gtk::{
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::Bytes,
    traits::ImageExt,
};
use shared::primitive::Size;
use tokio::runtime::Runtime;
use url::Url;

use crate::app::get_app_runtime;

pub struct BrowserTab {
    url: Url,
    is_active: bool,
}

impl BrowserTab {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            is_active: false,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn goto(&mut self, url: Url) {
        self.url = url;
        if self.is_active {
            get_app_runtime().update_state(|state| {
                state.paint_active_tab();
            });
        }
    }

    pub fn paint(&self, size: Size) {
        let html = std::fs::read_to_string(self.url.path.as_str()).expect("Unable to read HTML");
        let base_url = self.url.clone();

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let rendered_content = rt.block_on(render::render_once(html, base_url, size.clone()));
            let bytes = Bytes::from_owned(rendered_content);
            let pixbuf = Pixbuf::from_bytes(
                &bytes,
                Colorspace::Rgb,
                true,
                8,
                size.width as i32,
                size.height as i32,
                size.width as i32 * 4,
            );

            get_app_runtime().update_state(move |state| {
                state.ui.content_area.set_pixbuf(Some(&pixbuf));
            });
        });
    }
}
