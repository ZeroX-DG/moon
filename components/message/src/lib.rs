use ipc::{IpcError, Message};
use painting::DisplayList;
use rmpv::{
    decode::read_value,
    encode::write_value,
    ext::{from_value, to_value},
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

#[derive(Debug, Serialize, Deserialize)]
pub enum KernelMessage {
    LoadHTMLLocal(String),
    LoadCSSLocal(String),
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RendererMessage {
    RePaint(DisplayList),
    ResourceNotFound(String),
    Exit,
}

impl Message for KernelMessage {
    fn read(r: &mut impl BufRead) -> Result<Option<Self>, IpcError> {
        let value = read_value(r).map_err(|e| IpcError::Read(e.to_string()))?;
        let inner: KernelMessage =
            from_value(value).map_err(|e| IpcError::Deserialize(e.to_string()))?;
        log::debug!("<< Kernel {:?}", inner);
        let r = Some(inner);

        Ok(r)
    }

    fn write(self, w: &mut impl Write) -> Result<(), IpcError> {
        log::debug!(">> Kernel {:?}", self);
        let value = to_value(self).map_err(|e| IpcError::Serialize(e.to_string()))?;
        write_value(w, &value).map_err(|e| IpcError::Write(e.to_string()))?;
        w.flush().map_err(|e| IpcError::Write(e.to_string()))?;

        Ok(())
    }

    fn is_exit(&self) -> bool {
        match self {
            KernelMessage::Exit => true,
            _ => false,
        }
    }
}

impl Message for RendererMessage {
    fn read(r: &mut impl BufRead) -> Result<Option<Self>, IpcError> {
        let value = read_value(r).map_err(|e| IpcError::Read(e.to_string()))?;
        let inner: RendererMessage =
            from_value(value).map_err(|e| IpcError::Deserialize(e.to_string()))?;
        log::debug!("<< Renderer {:?}", inner);
        let r = Some(inner);

        Ok(r)
    }

    fn write(self, w: &mut impl Write) -> Result<(), IpcError> {
        log::debug!(">> Renderer {:?}", self);
        let value = to_value(self).map_err(|e| IpcError::Serialize(e.to_string()))?;
        write_value(w, &value).map_err(|e| IpcError::Write(e.to_string()))?;
        w.flush().map_err(|e| IpcError::Write(e.to_string()))?;

        Ok(())
    }

    fn is_exit(&self) -> bool {
        match self {
            RendererMessage::Exit => true,
            _ => false,
        }
    }
}
