use std::sync::{Arc, Mutex};

use crate::render_engine::RenderEngine;
use loader::ResourceLoader;
use shared::byte_string::ByteString;
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
        let mut render_engine = RenderEngine::new();

        let is_active = Arc::new(Mutex::new(false));
        let mut is_active_clone = is_active.clone();

        render_engine.on_new_bitmap(move |bitmap| {
            let is_active_clone = is_active_clone.clone();

            get_app_runtime().update_state(move |state| {
                let is_tab_active = is_active_clone.lock().unwrap();
                if *is_tab_active {
                    state.on_active_tab_bitmap(bitmap);
                }
            });
        });

        is_active_clone = is_active.clone();
        render_engine.on_new_title(move |title| {
            let is_active_clone = is_active_clone.clone();
            let title_clone = title.clone();

            get_app_runtime().update_state(move |state| {
                let is_tab_active = is_active_clone.lock().unwrap();
                if *is_tab_active {
                    state.ui.set_title(&title_clone);
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
        match self.url.scheme.as_str() {
            "http" | "https" | "file" => self.load_html(),
            "view-source" => self.load_source(),
            _ => self.load_not_supported(),
        }
    }

    fn load_html(&self) {
        let bytes = ResourceLoader::load(self.url.clone()).expect("Unable to load HTML");
        let html = ByteString::new(&bytes);
        self.render_engine
            .load_html(html.to_string(), self.url.clone());
    }

    fn load_source(&self) {
        let bytes = ResourceLoader::load(self.url.clone()).expect("Unable to load source");
        let raw_html_string = ByteString::new(&bytes).to_string();
        let raw_html = html_escape::encode_text(&raw_html_string);
        let source_html = format!("<html><pre>{}</pre></html>", raw_html);

        log::debug!("{}", source_html);

        self.render_engine.load_html(source_html, self.url.clone());
    }

    fn load_not_supported(&self) {
        let source_html = format!("<h1>Not supported protocol: {}</h1>", self.url.scheme);
        self.render_engine.load_html(source_html, self.url.clone());
    }
}
