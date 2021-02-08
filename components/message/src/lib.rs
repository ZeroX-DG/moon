use serde::{Deserialize, Serialize};
use ipc::{Message, IpcTransportError};
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageToRenderer {
    LoadHTMLLocal(String),
    LoadCSSLocal(String),
    SynAck(u16),
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageToKernel {
    RePaint(Vec<u8>),
    ResourceNotFound(String),
    Syn(u16),
    Ack(u16),
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserMessage {
    ToRenderer(MessageToRenderer),
    ToKernel(MessageToKernel)
}

impl Message for BrowserMessage {
    fn read(r: &mut impl BufRead) -> Result<Option<Self>, IpcTransportError> {
        let buf = match read_msg_bytes(r)
            .map_err(|e| IpcTransportError::Read(e))? {
            Some(b) => b,
            None => return Ok(None)
        };

        let msg = bincode::deserialize(&buf)
            .map_err(|e| IpcTransportError::Deserialize(e.to_string()))?;

        Ok(Some(msg))
    }

    fn write(self, w: &mut impl Write) -> Result<(), IpcTransportError> {
        let serialized = bincode::serialize(&self)
            .map_err(|e| IpcTransportError::Serialize(e.to_string()))?;

        write!(w, "{}\r\n\r\n", serialized.len())
            .map_err(|e| IpcTransportError::Write(e.to_string()))?;

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

fn read_msg_bytes(r: &mut impl BufRead) -> Result<Option<Vec<u8>>, String> {
    let mut buf = String::new();
    let mut size = None;

    loop {
        buf.clear();
        let read_count = r
            .read_line(&mut buf)
            .map_err(|e| e.to_string())?;

        if read_count == 0 {
            return Ok(None);
        }

        if !buf.ends_with("\r\n") {
            Err(format!("malformed header: {:?}", buf))?;
        }

        let buf = &buf[..buf.len() - 2];
        if buf.is_empty() {
            break;
        }

        size = Some(
            buf
                .parse::<usize>()
                .map_err(|_| "Failed to parse header size".to_owned())?,
        );
    }

    let size = size.ok_or("no Content-Length")?;
    let mut buf = buf.into_bytes();
    buf.resize(size, 0);
    r.read_exact(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(Some(buf))
}