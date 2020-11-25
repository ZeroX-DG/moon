use super::renderer_handler::{RendererHandler, RendererHandlers};
use flume::Sender;
use ipc::Selector;
use message::{KernelMessage, RendererMessage};

pub struct Kernel {
    renderer_handlers: RendererHandlers,
}

pub fn select(renderers: &[RendererHandler]) -> (usize, RendererMessage) {
    let mut selector = Selector::new();
    for (index, renderer) in renderers.iter().enumerate() {
        selector = selector.recv(renderer.receiver(), move |msg| {
            let msg = msg.expect("Error while selecting message");
            (index, msg)
        });
    }
    selector.wait()
}

impl Kernel {
    fn handle_renderer_msg(
        &mut self,
        index: usize,
        msg: RendererMessage,
        ui_sender: &Sender<painting::DisplayList>,
    ) {
        let handler = &self.renderer_handlers[index];
        match msg {
            RendererMessage::RePaint(display_list) => {
                ui_sender
                    .send(display_list)
                    .map_err(|e| log::error!("{:#?}", e.to_string()))
                    .unwrap();
            }
            _ => {}
        }
    }
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            renderer_handlers: RendererHandlers::new(),
        }
    }

    pub fn init_ui(&mut self) {
        let renderer = self.renderer_handlers.new_renderer();
        renderer
            .send(KernelMessage::LoadHTMLLocal(
                "/home/zerox/Desktop/Projects/moon/rendering/fixtures/test.html".to_string(),
            ))
            .unwrap();
        renderer
            .send(KernelMessage::LoadCSSLocal(
                "/home/zerox/Desktop/Projects/moon/rendering/fixtures/test.css".to_string(),
            ))
            .unwrap();
    }

    pub fn main_loop(&mut self, ui_sender: Sender<painting::DisplayList>) {
        loop {
            if self.renderer_handlers.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            let (index, msg) = select(self.renderer_handlers.inner());
            self.handle_renderer_msg(index, msg, &ui_sender);
        }
    }
}
