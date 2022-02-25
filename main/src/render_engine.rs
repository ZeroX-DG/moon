use flume::{bounded, Receiver, Sender};
use shared::primitive::Size;
use url::Url;

use render::{Renderer, RendererInitializeParams};

type Bitmap = Vec<u8>;

pub struct RenderEngine {
    renderer_action_tx: Sender<Box<dyn FnOnce(&mut Renderer) -> bool + Send>>,
    bitmap_rx: Receiver<Bitmap>,
}

impl RenderEngine {
    pub fn new() -> Self {
        let (renderer_action_tx, renderer_action_rx) =
            bounded::<Box<dyn FnOnce(&mut Renderer) -> bool + Send>>(1);

        let (bitmap_tx, bitmap_rx) = bounded::<Bitmap>(1);

        let _ = std::thread::spawn(move || {
            let mut renderer = Renderer::new();
            renderer.initialize(RendererInitializeParams {
                viewport: Size::new(1., 1.),
            });

            loop {
                match renderer_action_rx.recv() {
                    Ok(action) => {
                        let require_redraw = action(&mut renderer);
                        if require_redraw {
                            let bitmap = renderer.output();
                            bitmap_tx.send(bitmap).unwrap();
                        }
                    }
                    Err(_) => {
                        log::error!("Error while receiving renderer action");
                        break;
                    }
                }
            }
        });

        Self {
            renderer_action_tx,
            bitmap_rx,
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
            true
        });
    }

    pub fn on_new_bitmap(&self, handler: impl Fn(Bitmap) + Send + 'static) {
        let receiver = self.bitmap_rx.clone();
        let _ = std::thread::spawn(move || loop {
            match receiver.recv() {
                Ok(bitmap) => {
                    handler(bitmap);
                }
                _ => {
                    log::error!("Error while receiving bitmap");
                    break;
                }
            }
        });
    }

    fn update(&self, action: impl FnOnce(&mut Renderer) -> bool + 'static + Send) {
        self.renderer_action_tx.send(Box::new(action)).unwrap();
    }
}

