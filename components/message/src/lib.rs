use ipc::{Message, IpcError};
use std::io::{Write, Read};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MoonMessage {
    RePaint(Vec<u8>),
    Exit
}

impl Message for MoonMessage {
    fn read<R: Read>(r: &mut R) -> Result<Option<Self>, IpcError> {
        let mut buf = Vec::new();

        r.read_to_end(&mut buf)
            .map_err(|e| IpcError::Read(e.to_string()))?;

        match bincode::deserialize(&buf) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(IpcError::Deserialize(e.to_string()))
        }
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<(), IpcError> {
        let bindata = bincode::serialize(self)
            .map_err(|e| IpcError::Serialize(e.to_string()))?;

        w.write_all(&bindata)
            .map_err(|e| IpcError::Write(e.to_string()))
    }

    fn is_exit(&self) -> bool {
        match self {
            MoonMessage::Exit => true,
            _ => false
        }
    }
}
