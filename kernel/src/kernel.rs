use crate::UIAction;

use super::renderer::RendererHandler;
use flume::Sender;
use ipc::IpcMain;
use message::{BrowserMessage, MessageToKernel, MessageToRenderer};

pub struct Kernel {
    renderers: Vec<RendererHandler>,
}

#[derive(Debug)]
pub enum KernelError {
    SendError(String),
    Other(String),
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            renderers: Vec::new(),
        }
    }

    pub fn handle_msg(
        &mut self,
        reply: Sender<BrowserMessage>,
        msg: MessageToKernel,
        ipc: &IpcMain<BrowserMessage>,
        tx_ui: Sender<UIAction>,
    ) -> Result<(), KernelError> {
        match msg {
            MessageToKernel::RePaint(data) => tx_ui
                .send(UIAction::RePaint(data))
                .map_err(|e| KernelError::SendError(e.to_string()))?,

            MessageToKernel::ResourceNotFound(path) => Err(KernelError::Other(format!(
                "Resource not found: {:#?}",
                path
            )))?,

            MessageToKernel::Syn(id) => {
                log::info!("SYN received");
                reply
                    .send(BrowserMessage::ToRenderer(MessageToRenderer::SynAck(id)))
                    .map_err(|e| KernelError::SendError(e.to_string()))?;
                log::info!("SYN-ACK sent");
            }

            MessageToKernel::Ack(id) => {
                log::info!("ACK received");
                let renderer = &mut self.renderers[id as usize];
                renderer.set_connection(ipc.get_connection(id as usize));
            }
            _ => {}
        }

        Ok(())
    }

    pub fn new_tab(&mut self) -> usize {
        let id = self.renderers.len();
        let renderer = RendererHandler::new(id as u16);

        self.renderers.push(renderer);

        id
    }

    pub fn get_renderer(&self, id: usize) -> &RendererHandler {
        self.renderers.get(id).unwrap()
    }

    pub fn clean_up(&mut self) {
        self.renderers.clear();
    }
}
