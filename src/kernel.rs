use super::renderer::RendererHandler;
use flume::Sender;
use ipc::IpcMain;
use message::{BrowserMessage, MessageToKernel};

pub struct Kernel {
    renderers: Vec<RendererHandler>,
    pub ipc: IpcMain<BrowserMessage>
}

impl Kernel {
    fn handle_renderer_msg(
        &mut self,
        msg: BrowserMessage,
        ui_sender: &Sender<Vec<u8>>,
    ) {
        match msg {
            BrowserMessage::ToKernel(msg) => match msg {
                MessageToKernel::RePaint(data) => ui_sender.send(data).unwrap(),
                MessageToKernel::ResourceNotFound(path) => panic!("Resource not found: {:#?}", path),
                _ => {}
            }
            _ => {}
        }
    }
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            renderers: Vec::new(),
            ipc: IpcMain::new()
        }
    }

    pub fn spawn_new_renderer(&mut self) -> usize {
        let index = self.renderers.len();
        let handler = RendererHandler::new(index);
        self.renderers.push(handler);

        index
    }

    pub fn run(&mut self, ui_sender: Sender<Vec<u8>>) {
        self.ipc.run(4444);
        loop {
            let msg = self.ipc.receive();
            self.handle_renderer_msg(msg, &ui_sender);
        }
    }
}
