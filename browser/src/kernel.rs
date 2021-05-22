use super::renderer_client::RendererClient;
use super::window::UIMessage;
use super::messenger::Messenger;
use flume::Sender;
use message::*;
use ipc::IpcConnection;
use std::collections::HashMap;

pub type Connection = IpcConnection<BrowserMessage>;

#[derive(Debug)]
pub enum KernelError {
    ReceiveMessageError(String),
    SendMessageError(String),
}

pub struct Kernel {
    renderers: Vec<RendererClient>,
    renderer_connections: HashMap<Connection, usize>,
    window_sender: Sender<UIMessage>,
    messenger: Messenger,
    is_running: bool,
}

impl Kernel {
    pub fn new(window_sender: Sender<UIMessage>) -> Self {
        Self {
            renderers: vec![],
            renderer_connections: HashMap::new(),
            window_sender,
            messenger: Messenger::new(),
            is_running: false
        }
    }

    pub fn clean_up(&mut self) {
        for renderer in self.renderers.iter_mut() {
            renderer.disconnect();
        }
    }

    pub fn handle_request(
        &mut self,
        connection: Connection,
        request: RawRequest,
    ) -> Result<(), KernelError> {
        Ok(())
    }

    pub fn handle_response(
        &mut self,
        connection: Connection,
        request: RawResponse,
    ) -> Result<(), KernelError> {
        Ok(())
    }

    pub fn handle_notification(
        &mut self,
        connection: Connection,
        notification: RawNotification,
    ) -> Result<(), KernelError> {
        match notification.method.as_str() {
            Syn::METHOD => {
                Messenger::send_notification::<SynAck>(&connection, &())
            }
            Ack::METHOD => {
                if let Ok(params) = notification.cast::<Ack>() {
                    let client_index = self.renderers
                        .iter()
                        .position(|renderer| renderer.id() == &params.id);
                    
                    if let Some(client_index) = client_index {
                        self.renderers[client_index].set_ready(true);
                        self.renderer_connections.insert(connection, client_index);
                    } else {
                        log::warn!("Unable to ACK renderer with id: {}", params.id);
                    }
                }
                Ok(())
            }
            _ => Ok(())
        }
    }

    pub fn get_connection(&self, client_id: &str) -> Option<&Connection> {
        let client_index = match self.renderers
            .iter()
            .position(|renderer| renderer.id() == client_id) {
            Some(index) => index,
            None => return None
        };
        
        for (conn, index) in self.renderer_connections.iter() {
            if *index == client_index {
                return Some(conn);
            }
        }
        None
    }

    pub fn new_renderer(&mut self) -> &mut RendererClient {
        let client = RendererClient::new();
        self.renderers.push(client);

        let client = self.renderers.last_mut().unwrap();
        client
    }

    pub fn run_loop(&mut self) {
        self.is_running = true;

        loop {
            let handle_message_result = match self.messenger.receive() {

                Ok((conn, BrowserMessage::Request(req))) =>
                    self.handle_request(conn, req),

                Ok((conn, BrowserMessage::Response(res))) =>
                    self.handle_response(conn, res),

                Ok((conn, BrowserMessage::Notification(noti))) =>
                    self.handle_notification(conn, noti),

                Err(e) => Err(e)
            };

            if let Err(e) = handle_message_result {
                log::error!("Kernel panic when processing browser message:{:?}", e);
                break
            }
        }
    }
}

