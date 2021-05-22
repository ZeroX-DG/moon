use ipc::{IpcMain, IpcConnection};
use message::*;
use super::kernel::{Kernel, KernelError};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub type RawCallback =
    Box<dyn FnOnce(&mut Kernel, RawResponse) -> Result<(), KernelError>>;

pub struct Callback {
    pub id: u64,
    pub func: RawCallback,
}

type Connection = IpcConnection<BrowserMessage>;

pub struct Messenger {
    id: u64,
    ipc: IpcMain<BrowserMessage>,
}

impl Messenger {
    pub fn new() -> Self {
        let mut ipc = IpcMain::new();
        ipc.listen();

        Self {
            id: 0,
            ipc,
        }
    }

    fn send_message(connection: &Connection, message: BrowserMessage) -> Result<(), KernelError> {
        connection.sender
            .send(message)
            .map_err(|e| KernelError::SendMessageError(e.to_string()))?;
        Ok(())
    }

    pub fn send_notification<N: Notification>(
        connection: &Connection,
        params: &N::Params
    ) -> Result<(), KernelError> {
        let raw_notification = RawNotification::new::<N>(params);
        Self::send_message(connection, BrowserMessage::Notification(raw_notification))
    }

    pub fn send_request<R: Request>(
        connection: &mut Connection,
        params: &R::Params,
        callback: Box<dyn FnOnce(&mut Kernel, R::Result) -> Result<(), KernelError>>
    ) -> Result<(), KernelError>
    where
        R::Params: Serialize + Debug,
        R::Result: DeserializeOwned + 'static
    {
        log::debug!("Send request to kernel: {} with {:?}", R::METHOD, params);

        let id = connection.request_new_id();
        let request = RawRequest::new::<R>(id, params);
        let raw_callback: RawCallback =
            Box::new(move |kernel, raw_response: RawResponse| {
                log::debug!("{} callback", R::METHOD);
                let response = raw_response.cast::<R>()
                    .map_err(|e| Kernel::CastingResponseError(format!("Raw response: {:?}", e)))?;
                callback(kernel, response)
            });
        let func = Box::new(raw_callback);
        connection.add_callback(Callback { id, func });
        Self::send_message(connection, BrowserMessage::Request(request))
    }

    pub fn send_response_ok<R: Request>(
        connection: &Connection,
        request_id: u64,
        result: &R::Result
    ) -> Result<(), KernelError> {
        let raw_response = RawResponse::ok::<R>(request_id, result);
        Self::send_message(connection, BrowserMessage::Response(raw_response))
    }

    pub fn receive(&self) -> Result<(Connection, BrowserMessage), KernelError> {
        self.ipc
            .receive()
            .map_err(|e| KernelError::ReceiveMessageError(format!("{:?}", e)))
    }
}

