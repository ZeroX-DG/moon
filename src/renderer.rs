use ipc::{Client, RecvError, SendError};
use std::process::{Command, Stdio, Child};
use message::{KernelMessage, RendererMessage};

pub struct Renderers {
    renderers: Vec<RendererHandler>
}

pub struct RendererHandler {
    process: Child,
    client: Client<RendererMessage, KernelMessage>
}

impl Renderers {
    pub fn new() -> Self {
        Self {
            renderers: Vec::new()
        }
    }

    pub fn new_renderer(&mut self) -> &RendererHandler {
        self.renderers.push(RendererHandler::new());
        self.renderers.last().unwrap()
    }

    pub fn close_all(&mut self) {
        self.renderers.iter_mut().for_each(|renderer| renderer.close());
        self.renderers.clear();
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
            client: Client::new(|| process_stdout, || process_stdin)
        }
    }

    pub fn close(&mut self) {
        self.send(KernelMessage::Exit)
            .expect("Unable to send exit message to renderer");

        // delay a bit so the exit message get sent
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.process.kill()
            .expect("Unable to kill renderer");
    }

    pub fn recv(&self) -> Result<RendererMessage, RecvError> {
        self.client.receiver.recv()
    }

    pub fn send(&self, msg: KernelMessage) -> Result<(), SendError<KernelMessage>> {
        self.client.sender.send(msg)
    }
}
