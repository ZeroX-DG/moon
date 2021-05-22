use super::client::{Client, Message};
use flume::{Sender, Receiver};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct IpcConnection<M: Message> {
    pub id: String,
    pub sender: Sender<M>,
    pub receiver: Receiver<M>,
}

impl<M: Message> From<&Client<M>> for IpcConnection<M> {
    fn from(client: &Client<M>) -> Self {
        Self {
            id: client.id().to_string(),
            sender: client.sender().clone(),
            receiver: client.receiver().clone()
        }
    }
}

impl<M: Message> Hash for IpcConnection<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<M: Message> PartialEq for IpcConnection<M> {
    fn eq(&self, other: &IpcConnection<M>) -> bool {
        self.id == other.id
    }
}

impl<M: Message> Eq for IpcConnection<M> {}

