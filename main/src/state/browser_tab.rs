use std::sync::{Arc, Mutex};

use crate::render_client::RenderClient;
use loader::ResourceLoader;
use render::OutputEvent;
use shared::byte_string::ByteString;
use shared::primitive::Size;
use url::Url;

use crate::app::get_app_runtime;

pub struct BrowserTab {
    url: Url,
    is_active: Arc<Mutex<bool>>,
    client: RenderClient,
}

impl BrowserTab {
    pub fn new(url: Url) -> Self {
        let client = RenderClient::new();
        let is_active = Arc::new(Mutex::new(false));
        let is_active_clone = is_active.clone();

        let events = client.events();

        let _ = std::thread::spawn(move || loop {
            let event = events.recv().expect("Render Engine disconnected");
            let is_active = is_active_clone.clone();

            match event {
                OutputEvent::FrameRendered(bitmap) => {
                    get_app_runtime().update_state(move |state| {
                        let is_tab_active = is_active.lock().unwrap();
                        if *is_tab_active {
                            state.on_active_tab_bitmap(bitmap);
                        }
                    });
                }
            }
        });

        Self {
            url,
            is_active,
            client,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        *self.is_active.lock().unwrap() = active;
    }

    pub fn resize(&self, size: Size) {
        self.client.resize(size);
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn goto(&mut self, url: Url) {
        self.url = url;
        self.load();
    }

    pub fn load(&self) {
        self.update_current_url();
        match self.url.scheme.as_str() {
            "http" | "https" | "file" => self.load_html(),
            "view-source" => self.load_source(),
            _ => self.load_not_supported(),
        }
    }

    pub fn load_error(&self, title: &str, error: &str) {
        let source_html = self.get_error_page_content(title, error);
        self.client.load_html(source_html, self.url.clone());
    }

    fn update_current_url(&self) {
        get_app_runtime().update_state(|state| {
            let active_tab_url = state.active_tab().url().as_str();
            state.ui.set_url(&active_tab_url);
        });
    }

    fn get_error_page_content(&self, title: &str, error: &str) -> String {
        format!(
            "
            <html>
                <style>
                    body {{ background-color: #262ded }}
                    #error-content {{
                        width: 500px;
                        margin: 0 auto;
                        margin-top: 50px;
                        color: white;
                    }}
                </style>
                <div id='error-content'>
                    <h1>{}</h1>
                    <p>{}</p>
                </div>
            </html>
        ",
            title, error
        )
    }

    fn load_html(&self) {
        match ResourceLoader::load(self.url.clone()) {
            Ok(bytes) => {
                let html = ByteString::new(&bytes);
                self.client.load_html(html.to_string(), self.url.clone());
            }
            Err(e) => {
                self.load_error("Aw, Snap!", &e.get_friendly_message());
            }
        }
    }

    fn load_source(&self) {
        match ResourceLoader::load(self.url.clone()) {
            Ok(bytes) => {
                let raw_html_string = ByteString::new(&bytes).to_string();
                let raw_html = html_escape::encode_text(&raw_html_string);
                let source_html = format!("<html><pre>{}</pre></html>", raw_html);

                self.client.load_html(source_html, self.url.clone());
            }
            Err(e) => {
                self.load_error("Aw, Snap!", &e.get_friendly_message());
            }
        }
    }

    fn load_not_supported(&self) {
        let error = format!(
            "Unable to load resource via unsupported protocol: {}",
            self.url.scheme
        );
        self.load_error("Unsupported Protocol", &error);
    }
}
