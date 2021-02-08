use crate::UIAction;

use super::renderer::RendererHandler;
use flume::Sender;
use ipc::IpcMain;
use message::{BrowserMessage, MessageToKernel, MessageToRenderer};

pub struct Kernel {
    renderers: Vec<RendererHandler>
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            renderers: Vec::new()
        }
    }

    pub fn handle_msg(
        &mut self,
        reply: Sender<BrowserMessage>,
        msg: MessageToKernel,
        ipc: &IpcMain<BrowserMessage>,
        tx_ui: Sender<UIAction>,
    ) {
        match msg {
            MessageToKernel::RePaint(data) => {
                // println!("{:#?}", data);
                tx_ui.send(UIAction::RePaint(data)).unwrap();
            },
            MessageToKernel::ResourceNotFound(path) => panic!("Resource not found: {:#?}", path),
            MessageToKernel::Syn(id) => {
                println!("SYN received");
                reply.send(BrowserMessage::ToRenderer(MessageToRenderer::SynAck(id)))
                    .expect("Unable to reply Syn");
                println!("SYN-ACK sent");
            }
            MessageToKernel::Ack(id) => {
                println!("ACK received");
                let renderer = &mut self.renderers[id as usize];
                renderer.set_connection(ipc.get_connection(id as usize));
            }
            _ => {}
        }
    }

    pub fn new_tab(&mut self) -> usize {
        let id = self.renderers.len();
        let renderer = RendererHandler::new(id);

        self.renderers.push(renderer);

        id
    }

    pub fn get_renderer(&self, id: usize) -> &RendererHandler {
        self.renderers.get(id).unwrap()
    }
}
