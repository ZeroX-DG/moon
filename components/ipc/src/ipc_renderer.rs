use super::client::{Client, Message};
use super::net::Stream;
use std::ops::Deref;

pub struct IpcRenderer<M: Message> {
    client: Client<M>,
}

impl<M: Message> IpcRenderer<M> {
    pub fn new() -> Self {
        let (stream_read, stream_write) = loop {
            if let Ok(stream_read) = Stream::connect() {
                let stream_write = stream_read
                    .try_clone()
                    .expect("Unable to obtain write stream");
                break (stream_read, stream_write);
            }
        };

        Self {
            client: Client::new(|| stream_read, || stream_write),
        }
    }
}

impl<M: Message> Deref for IpcRenderer<M> {
    type Target = Client<M>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
