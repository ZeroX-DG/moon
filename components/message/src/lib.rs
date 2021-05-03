mod request;
mod notification;

use ipc::{IpcTransportError, Message};
use serde::{Deserialize, Serialize};
use notification::{Notification, Exit};
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserMessage {
    Request(RawRequest),
    Response(RawResponse),
    Notification(RawNotification),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawRequest {
    pub id: u64,
    pub method: String,

    // Bytes after serialized by bincode
    pub params: Vec<u8>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawResponse {
    pub request_id: u64,

    // Bytes to be deserialize by bincode
    pub result: Option<Vec<u8>>,
    pub error: Option<Vec<u8>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawNotification {
    pub method: String,
    pub params: Vec<u8>
}

impl Message for BrowserMessage {
    fn read(r: &mut impl BufRead) -> Result<Option<Self>, IpcTransportError> {
        let buf = match read_msg_bytes(r).map_err(|e| IpcTransportError::Read(e))? {
            Some(b) => b,
            None => return Ok(None),
        };

        let msg = bincode::deserialize(&buf)
            .map_err(|e| IpcTransportError::Deserialize(e.to_string()))?;

        Ok(Some(msg))
    }

    fn write(self, w: &mut impl Write) -> Result<(), IpcTransportError> {
        let serialized =
            bincode::serialize(&self).map_err(|e| IpcTransportError::Serialize(e.to_string()))?;

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
            BrowserMessage::Notification(n) => n.is::<Exit>(),
            _ => false
        }
    }
}

impl RawNotification {
    pub fn is<N>(&self) -> bool
    where
        N: Notification
    {
        self.method == N::METHOD
    }
}

fn read_msg_bytes(r: &mut impl BufRead) -> Result<Option<Vec<u8>>, String> {
    let mut buf = String::new();
    let mut size = None;

    loop {
        buf.clear();
        let read_count = r.read_line(&mut buf).map_err(|e| e.to_string())?;

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
            buf.parse::<usize>()
                .map_err(|_| "Failed to parse header size".to_owned())?,
        );
    }

    let size = size.ok_or("no Content-Length")?;
    let mut buf = buf.into_bytes();
    buf.resize(size, 0);
    r.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(Some(buf))
}
