mod client;
use client::Client;
use std::{net::{TcpListener, TcpStream, SocketAddr}, thread};
use std::sync::{Arc, Mutex};
use std::ops::Deref;
use flume::{Receiver, Selector, Sender};

pub use client::{Message, IpcTransportError};

pub struct IpcMain<M: Message> {
    clients: Arc<Mutex<Vec<Client<M>>>>
}

pub struct IpcRenderer<M: Message> {
    client: Client<M>
}

pub struct IpcConnection<M> {
    pub sender: Sender<M>,
    pub receiver: Receiver<M>
}

impl<M: Message> IpcMain<M> {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn run(&mut self, port: u16) {
        let clients = self.clients.clone();

        thread::spawn(move || {
            let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
                .expect("Unable to bind port");
    
            for stream in listener.incoming() {
                let stream_read = stream.expect("Unable to obtain read stream");
                let stream_write = stream_read.try_clone().expect("Unable to obtain write stream");
                let client = Client::<M>::new(|| stream_read, || stream_write);

                clients.lock().unwrap().push(client);
            }
            
        });
    }

    pub fn receive(&mut self) -> M {
        let clients = &*self.clients.lock().unwrap();
        let mut selector = Selector::new();

        for renderer in clients {
            selector = selector.recv(renderer.receiver(), |msg| msg);
        }

        selector.wait().unwrap()
    }

    pub fn get_connection(&self, index: usize) -> IpcConnection<M> {
        let clients = self.clients.lock().unwrap();
        let client = &clients[index];

        IpcConnection {
            sender: client.sender.clone(),
            receiver: client.receiver.clone()
        }
    }
}

impl<M: Message> IpcRenderer<M> {
    pub fn new(port: u16) -> Self {
        let stream_read = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], port)))
            .expect("Unable to obtain read stream");
        let stream_write = stream_read.try_clone()
            .expect("Unable to obtain write stream");

        Self {
            client: Client::new(|| stream_read, || stream_write)
        }
    }
}

impl<M: Message> Deref for IpcRenderer<M> {
    type Target = Client<M>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}