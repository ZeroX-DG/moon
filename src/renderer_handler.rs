use ipc::{Client, Receiver, RecvError, SendError, Sender};
use message::{KernelMessage, RendererMessage};
use std::ops::Deref;
use std::process::{Child, Command, Stdio};

pub struct RendererHandlers {
    handlers: Vec<RendererHandler>,
}

pub struct RendererHandler {
    process: Child,
    client: Client<RendererMessage, KernelMessage>,
}

impl Deref for RendererHandlers {
    type Target = Vec<RendererHandler>;
    fn deref(&self) -> &Self::Target {
        &self.handlers
    }
}

impl RendererHandlers {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn new_renderer(&mut self) -> &RendererHandler {
        self.handlers.push(RendererHandler::new());
        self.handlers.last().unwrap()
    }

    pub fn inner(&self) -> &[RendererHandler] {
        &self.handlers
    }
}

impl RendererHandler {
    pub fn new() -> Self {
        let mut process = Command::new("target/debug/rendering")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Unable to start renderer");

        let process_stdin = process.stdin.take().unwrap();
        let process_stdout = process.stdout.take().unwrap();

        Self {
            process,
            client: Client::new(|| process_stdout, || process_stdin),
        }
    }

    pub fn receiver(&self) -> &Receiver<RendererMessage> {
        &self.client.receiver
    }

    pub fn sender(&self) -> &Sender<KernelMessage> {
        &self.client.sender
    }

    pub fn recv(&self) -> Result<RendererMessage, RecvError> {
        self.receiver().recv()
    }

    pub fn send(&self, msg: KernelMessage) -> Result<(), SendError<KernelMessage>> {
        self.sender().send(msg)
    }
}
