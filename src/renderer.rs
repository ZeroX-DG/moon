use ipc::IpcConnection;
use message::BrowserMessage;
use std::process::{Child, Command};
use std::env;

pub struct RendererHandler {
    process: Child,
    connection: Option<IpcConnection<BrowserMessage>>,
}

impl RendererHandler {
    pub fn new(id: u16) -> Self {
        let mut dir = env::current_exe().expect("Unable to obtain current path");
        dir.pop();
        dir.push("rendering");

        let process = Command::new(dir)
            .args(&["--id", &id.to_string()])
            .spawn()
            .expect("Unable to start renderer");

        Self {
            process,
            connection: None,
        }
    }

    pub fn set_connection(&mut self, conn: IpcConnection<BrowserMessage>) {
        self.connection = Some(conn);
    }

    pub fn is_ready(&self) -> bool {
        self.connection.is_some()
    }

    pub fn send(&self, msg: BrowserMessage) {
        if let Some(conn) = &self.connection {
            conn.sender.send(msg).unwrap();
        }
    }
}

impl Drop for RendererHandler {
    fn drop(&mut self) {
        self.process.kill().unwrap();
    }
}
