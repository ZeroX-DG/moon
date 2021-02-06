use std::process::{Child, Command};
use message::BrowserMessage;

use super::kernel::Kernel;

pub struct RendererHandler {
    process: Child,
    id: usize
}

impl RendererHandler {
    pub fn new(id: usize) -> Self {
        let process = Command::new("target/debug/rendering")
            .spawn()
            .expect("Unable to start renderer");
        
        Self {
            process,
            id
        }
    }

    pub fn id(&self) -> &usize {
        &self.id
    }
}