use ipc::IpcRenderer;
use message::{BrowserMessage, RawResponse, Request, RawRequest};
use super::renderer::{Renderer, RendererError};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub type RawCallback =
    Box<dyn FnOnce(&mut Renderer, RawResponse) -> Result<(), RendererError>>;

pub struct Callback {
    pub id: u64,
    pub func: RawCallback,
}

pub struct Messenger {
    id: u64,
    ipc: IpcRenderer<BrowserMessage>,
    callbacks: Vec<Callback>
}

impl Messenger {
    pub fn new() -> Self {
        Self {
            id: 0,
            ipc: IpcRenderer::new(),
            callbacks: Vec::new()
        }
    }

    pub fn get_callback(&mut self, id: u64) -> Option<Callback> {
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

    fn request_new_id(&mut self) -> u64 {
        self.id += 1;
        self.id
    }

    fn send_message(&mut self, message: BrowserMessage) -> Result<(), RendererError> {
        self.ipc.sender()
            .send(message)
            .map_err(|_| RendererError::KernelDisconnected)?;
        Ok(())
    }

    pub fn send_request<R: Request>(
        &mut self,
        params: &R::Params,
        callback: Box<dyn FnOnce(&mut Renderer, R::Result) -> Result<(), RendererError>>
    ) -> Result<(), RendererError>
    where
        R::Params: Serialize + Debug,
        R::Result: DeserializeOwned + 'static
    {
        log::debug!("Send request to kernel: {} with {:?}", R::METHOD, params);

        let id = self.request_new_id();
        let request = RawRequest::new::<R>(id, params);
        let raw_callback: RawCallback =
            Box::new(move |renderer, raw_response: RawResponse| {
                log::debug!("{} callback", R::METHOD);
                let response = raw_response.cast::<R>()
                    .map_err(|e| RendererError::CastingResponseError(format!("Raw response: {:?}", e)))?;
                callback(renderer, response)
            });
        let func = Box::new(raw_callback);
        self.callbacks.push(Callback { id, func });
        self.send_message(BrowserMessage::Request(request))
    }

    pub fn send_response_ok<R: Request>(
        &mut self,
        request_id: u64,
        result: &R::Result
    ) -> Result<(), RendererError> {
        let raw_response = RawResponse::ok::<R>(request_id, result);
        self.send_message(BrowserMessage::Response(raw_response))
    }

    pub fn receive(&mut self) -> Result<BrowserMessage, RendererError> {
        self.ipc.receiver()
            .recv()
            .map_err(|e| RendererError::ReceiveMessageError(e.to_string()))
    }
}
