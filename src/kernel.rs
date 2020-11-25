use super::renderer_handler::{RendererHandlers, RendererHandler};
use message::{RendererMessage, KernelMessage};
use ipc::Selector;
use flume::Sender;

pub struct Kernel {
    renderer_handlers: RendererHandlers
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
        ui_sender: &Sender<painting::DisplayList>
    ) {
        let handler = &self.renderer_handlers[index];
        match msg {
            RendererMessage::RePaint(display_list) => {
                ui_sender.send(display_list);
            }
            _ => {}
        }
    }
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            renderer_handlers: RendererHandlers::new()
        }
    }

    pub fn renderer_handlers(&mut self) -> &mut RendererHandlers {
        &mut self.renderer_handlers
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

    pub fn clean_up(&mut self) {
        self.renderer_handlers.close_all();
    }
}
