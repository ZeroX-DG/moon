use flume::{Receiver, Sender};
use shared::primitive::Size;

use render::{InputEvent, OutputEvent, RenderEngine};

pub struct RenderClient {
    event_sender: Sender<InputEvent>,
    event_receiver: Receiver<OutputEvent>,
    ready_receiver: Receiver<()>,
}

impl RenderClient {
    pub fn new() -> Self {
        let (render_input_tx, render_input_rx) = flume::unbounded();
        let (render_output_tx, render_output_rx) = flume::unbounded();

        let (ready_tx, ready_rx) = flume::bounded(1);

        // spawn a new thread to run render engine
        let _ = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let render_engine = RenderEngine::new(Size::new(1., 1.)).await;

                ready_tx.send(()).unwrap();

                // run render engine (this is an infinite loop)
                if let Err(e) = render_engine.run(render_input_rx, render_output_tx).await {
                    log::error!("Render Engine exited with error: {}", e.to_string());
                }
            });
        });

        Self {
            event_sender: render_input_tx,
            event_receiver: render_output_rx,
            ready_receiver: ready_rx,
        }
    }

    pub fn wait_till_ready(&self) {
        self.ready_receiver
            .recv()
            .expect("Error while waiting for render client to be ready")
    }

    pub fn events(&self) -> Receiver<OutputEvent> {
        self.event_receiver.clone()
    }

    pub fn resize(&self, size: Size) {
        self.event_sender
            .send(InputEvent::ViewportResize(size))
            .expect("Unable to resize");
    }

    pub fn scroll(&self, y: f32) {
        self.event_sender
            .send(InputEvent::Scroll(y))
            .expect("Unable to send scroll event");
    }

    pub fn load_raw_url(&self, url: String) {
        self.event_sender
            .send(InputEvent::LoadRawURL(url))
            .expect("Unable to load URL");
    }

    pub fn mouse_move(&self, coord: shared::primitive::Point) {
        self.event_sender
            .send(InputEvent::MouseMove(coord))
            .expect("Unable to send mouse move event");
    }

    pub fn reload(&self) {
        self.event_sender
            .send(InputEvent::Reload)
            .expect("Unable to send mouse move event");
    }
}
