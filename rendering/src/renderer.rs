use super::page::Page;
use super::paint::Painter;
use ipc::IpcRenderer;
use message::{BrowserMessage, RawResponse};

#[derive(Debug)]
pub enum RendererError {
    ReceiveMessageError(String)
}

pub type RawCallback =
    Box<dyn FnOnce(&mut Renderer, RawResponse) -> Result<(), RendererError>>;

pub struct Callback {
    pub id: u64,
    pub func: RawCallback,
}

pub struct Renderer {
    page: Page,
    painter: Painter,
    ipc: IpcRenderer<BrowserMessage>,
    callbacks: Vec<Callback>
}

impl Renderer {
    pub async fn new() -> Self {
        Self {
            page: Page::new(),
            painter: Painter::new().await,
            ipc: IpcRenderer::new(),
            callbacks: Vec::new()
        }
    }

    fn get_callback(&mut self, id: u64) -> Option<Callback> {
        let cb_index = self.callbacks
            .iter()
            .position(|cb| cb.id == id);

        if let Some(index) = cb_index {
            let callback = self.callbacks.swap_remove(index);
            Some(callback)
        } else {
            None
        }
    }

    fn handle_response(&mut self, raw_response: RawResponse) -> Result<(), RendererError> {
        if let Some(callback) = self.get_callback(raw_response.request_id) {
            return (callback.func)(self, raw_response);
        }
        Ok(())
    }

    pub fn run_event_loop(&mut self) {
        loop {
            let result = match self.ipc.receiver().recv() {
                Ok(BrowserMessage::Request(request)) => Ok(()),
                Ok(BrowserMessage::Response(raw_response)) => self.handle_response(raw_response),
                Ok(BrowserMessage::Notification(raw_notification)) => Ok(()),
                Err(e) => Err(RendererError::ReceiveMessageError(e.to_string()))
            };

            if let Err(e) = result {
                log::error!("Renderer error encountered: {:?}", e);
                break
            }
        }
    }
}
