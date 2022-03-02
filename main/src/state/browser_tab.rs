use std::sync::{Arc, Mutex};

use crate::render_engine::RenderEngine;
use loader::ResourceLoader;
use shared::primitive::Size;
use url::Url;

use crate::app::get_app_runtime;

pub struct BrowserTab {
    url: Url,
    is_active: Arc<Mutex<bool>>,
    render_engine: RenderEngine,
}

impl BrowserTab {
    pub fn new(url: Url) -> Self {
        let render_engine = RenderEngine::new();

        let is_active = Arc::new(Mutex::new(false));
        let is_active_clone = is_active.clone();

        render_engine.on_new_bitmap(move |bitmap| {
            let is_active_clone = is_active_clone.clone();

            get_app_runtime().update_state(move |state| {
                let is_tab_active = is_active_clone.lock().unwrap();
                if *is_tab_active {
                    state.on_active_tab_bitmap(bitmap);
                }
            });
        });

        Self {
            url,
            is_active,
            render_engine,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        *self.is_active.lock().unwrap() = active;
    }

    pub fn resize(&self, size: Size) {
        self.render_engine.resize(size);
    }

    pub fn goto(&mut self, url: Url) {
        self.url = url;
        self.load();
    }

    pub fn load(&self) {
        let bytes = ResourceLoader::load(self.url.clone()).expect("Unable to load HTML");
        let html = String::from_utf8(bytes).unwrap();
        self.render_engine.load_html(html, self.url.clone());
    }
}
