use gtk::{gdk_pixbuf::{Colorspace, Pixbuf}, glib::Bytes, traits::ImageExt};
use shared::primitive::Size;
use tokio::runtime::Runtime;
use url::Url;

use crate::app::get_app_runtime;

pub struct BrowserTab {
    url: Url
}

impl BrowserTab {
    pub fn new(url: Url) -> Self {
        Self {
            url
        }
    }

    pub fn paint(&self, size: Size) {
        let html = std::fs::read_to_string(self.url.path.as_str()).expect("Unable to read HTML");
        let render_size = (size.width as u32, size.height as u32);
        let base_url = self.url.clone();

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let rendered_content = rt.block_on(render::render_once(html, base_url, render_size));
            let bytes = Bytes::from_owned(rendered_content);
            let pixbuf = Pixbuf::from_bytes(
                &bytes,
                Colorspace::Rgb,
                true,
                8,
                size.width as i32,
                size.height as i32,
                size.width as i32 * 4
            );
            
            get_app_runtime().update_state(move |state| {
                state.ui.content_area.set_pixbuf(Some(&pixbuf));
            });
        });
    }
}
