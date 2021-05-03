use flume::{RecvError, Selector, Sender};
use super::client::{Client, Message};
use super::net::Listener;
use std::sync::{Arc, Mutex};
use std::thread;

pub enum IpcMainReceiveError {
    NoConnections,
    Other(RecvError),
}

pub struct IpcMain<M: Message> {
    clients: Arc<Mutex<Vec<Client<M>>>>,
}

impl<M: Message> IpcMain<M> {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn listen(&mut self) {
        let clients = self.clients.clone();

        thread::spawn(move || {
            let listener = Listener::bind().expect("Unable to bind listener");

            for stream in listener.incoming() {
                let stream_read = stream.expect("Unable to obtain read stream");
                let stream_write = stream_read
                    .try_clone()
                    .expect("Unable to obtain write stream");
                let client = Client::<M>::new(|| stream_read, || stream_write);

                clients.lock().unwrap().push(client);
            }
        });
    }

    pub fn receive(&self) -> Result<(Sender<M>, M), IpcMainReceiveError> {
        let clients = &*self.clients.lock().unwrap();

        if clients.len() == 0 {
            return Err(IpcMainReceiveError::NoConnections);
        }

        let mut selector = Selector::new();

        for (index, renderer) in clients.iter().enumerate() {
            let index = index.clone();
            selector = selector.recv(renderer.receiver(), move |msg| (index, msg));
        }

        let (index, msg) = selector.wait();
        let msg = msg.map_err(|e| IpcMainReceiveError::Other(e))?;

        Ok((clients[index].sender().clone(), msg))
    }
}
