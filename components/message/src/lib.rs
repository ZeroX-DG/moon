use serde::{Deserialize, Serialize};
use ipc::{Message, IpcTransportError};
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageToRenderer {
    LoadHTMLLocal(String),
    LoadCSSLocal(String),
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageToKernel {
    RePaint(Vec<u8>),
    ResourceNotFound(String),
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserMessage {
    ToRenderer(MessageToRenderer),
    ToKernel(MessageToKernel)
}

impl Message for BrowserMessage {
    fn read(r: &mut impl BufRead) -> Result<Option<Self>, IpcTransportError> {
        let mut buf = Vec::new();
        let read_count = r
            .read_to_end(&mut buf)
            .map_err(|e| IpcTransportError::Read(e.to_string()))?;

        if read_count == 0 {
            return Ok(None);
        }

        let msg = bincode::deserialize(&buf)
            .map_err(|e| IpcTransportError::Deserialize(e.to_string()))?;

        Ok(Some(msg))
    }

    fn write(self, w: &mut impl Write) -> Result<(), IpcTransportError> {
        let serialized = bincode::serialize(&self)
            .map_err(|e| IpcTransportError::Serialize(e.to_string()))?;

        w.write_all(&serialized)
            .map_err(|e| IpcTransportError::Write(e.to_string()))?;

        w.flush()
            .map_err(|e| IpcTransportError::Write(e.to_string()))?;

        Ok(())
    }
    
    fn is_exit(&self) -> bool {
        match self {
            BrowserMessage::ToKernel(msg) => match msg {
                MessageToKernel::Exit => true,
                _ => false
            },
            BrowserMessage::ToRenderer(msg) => match msg {
                MessageToRenderer::Exit => true,
                _ => false
            }
        }
    }
}