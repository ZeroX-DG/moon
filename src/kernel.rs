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
        ipc: &IpcMain<BrowserMessage>
    ) {
        match msg {
            MessageToKernel::RePaint(data) => {
                println!("{:#?}", data);
                //ui_sender.send(data).unwrap();
            },
            MessageToKernel::ResourceNotFound(path) => panic!("Resource not found: {:#?}", path),
            MessageToKernel::Syn(id) => {
                reply.send(BrowserMessage::ToRenderer(MessageToRenderer::SynAck(id)))
                    .expect("Unable to reply Syn");
            }
            MessageToKernel::Ack(id) => {
                let renderer = &mut self.renderers[id as usize];
                renderer.set_connection(ipc.get_connection(id as usize));
            }
            _ => {}
        }
    }
}
