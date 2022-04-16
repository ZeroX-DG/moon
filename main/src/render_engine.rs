use std::sync::{Arc, Mutex};

use flume::{bounded, RecvError, Sender};
use shared::primitive::Size;
use url::Url;

use render::{Renderer, RendererInitializeParams};

type Bitmap = Vec<u8>;

pub enum RenderEngineData {
    Bitmap(Bitmap),
    Title(String),
}

pub struct RenderEngine {
    kernel_action_tx: Sender<Box<dyn FnOnce(&mut Renderer) -> bool + Send>>,
    new_bitmap_handler: Arc<Mutex<Option<Box<dyn Fn(Bitmap) + Send>>>>,
    new_title_handler: Arc<Mutex<Option<Box<dyn Fn(String) + Send>>>>,
}

impl RenderEngine {
    pub fn new() -> Self {
        let (kernel_action_tx, kernel_action_rx) =
            bounded::<Box<dyn FnOnce(&mut Renderer) -> bool + Send>>(1);

        let (window_action_tx, window_action_rx) = bounded::<RenderEngineData>(1);

        let _ = std::thread::spawn(move || {
            let mut renderer = Renderer::new();
            renderer.initialize(RendererInitializeParams {
                viewport: Size::new(1., 1.),
            });

            let window_action_tx_clone = window_action_tx.clone();

            renderer.on_new_title(move |title| {
                window_action_tx_clone
                    .send(RenderEngineData::Title(title))
                    .unwrap();
            });

            loop {
                flume::Selector::new()
                    .recv(
                        &kernel_action_rx,
                        |action: Result<
                            Box<dyn FnOnce(&mut Renderer) -> bool + Send>,
                            RecvError,
                        >| match action {
                            Ok(action) => {
                                let require_redraw = action(&mut renderer);
                                if require_redraw {
                                    let bitmap = renderer.output();
                                    window_action_tx
                                        .send(RenderEngineData::Bitmap(bitmap))
                                        .unwrap();
                                }
                            }
                            Err(_) => {
                                panic!("Error while receiving renderer action");
                            }
                        },
                    )
                    .wait();
            }
        });

        let new_bitmap_handler: Arc<Mutex<Option<Box<dyn Fn(Bitmap) + Send>>>> =
            Arc::new(Mutex::new(None));
        let new_title_handler: Arc<Mutex<Option<Box<dyn Fn(String) + Send>>>> =
            Arc::new(Mutex::new(None));

        let new_bitmap_handler_clone = new_bitmap_handler.clone();
        let new_title_handler_clone = new_title_handler.clone();

        let _ = std::thread::spawn(move || loop {
            match window_action_rx.recv() {
                Ok(RenderEngineData::Bitmap(bitmap)) => {
                    if let Some(handler) = &*new_bitmap_handler_clone.lock().unwrap() {
                        handler(bitmap);
                    }
                }
                Ok(RenderEngineData::Title(title)) => {
                    if let Some(handler) = &*new_title_handler_clone.lock().unwrap() {
                        handler(title);
                    }
                }
                _ => {
                    log::error!("Error while receiving bitmap");
                    break;
                }
            }
        });

        Self {
            kernel_action_tx,
            new_bitmap_handler,
            new_title_handler,
        }
    }

    pub fn load_html(&self, html: String, base_url: Url) {
        self.update(|renderer| {
            renderer.load_html(html, base_url);
            true
        });
    }

    pub fn resize(&self, size: Size) {
        self.update(|renderer| {
            renderer.resize(size);
            renderer.paint();
            true
        });
    }

    pub fn on_new_bitmap(&mut self, handler: impl Fn(Bitmap) + Send + 'static) {
        self.new_bitmap_handler
            .lock()
            .unwrap()
            .replace(Box::new(handler));
    }

    pub fn on_new_title(&self, handler: impl Fn(String) + Send + 'static) {
        self.new_title_handler
            .lock()
            .unwrap()
            .replace(Box::new(handler));
    }

    fn update(&self, action: impl FnOnce(&mut Renderer) -> bool + 'static + Send) {
        self.kernel_action_tx.send(Box::new(action)).unwrap();
    }
}
