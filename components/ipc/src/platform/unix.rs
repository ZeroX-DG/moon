use crate::IpcError;
use bincode;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use message::{KernelMessage, RendererMessage};

const IPC_ADDRESS: &str = "/tmp/moon-ipc";

pub struct IpcMain {
    streams: Arc<Mutex<Vec<UnixStream>>>,
    active_stream_index: usize
}

pub struct IpcRenderer{
    stream: UnixStream
}

impl IpcMain {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(Mutex::new(Vec::new())),
            active_stream_index: 0,
        }
    }

    pub fn init(&mut self) {
        let listener = UnixListener::bind(IPC_ADDRESS).expect("Unable to create IPC main");

        let streams = self.streams.clone();

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        stream.set_read_timeout(None).unwrap();
                        streams.lock().unwrap().push(stream);
                    }
                    Err(err) => {
                        log::error!("IpcError(Unix): {}", err);
                        break;
                    }
                }
            }
        });
    }

    pub fn send(&self, data: KernelMessage) -> Result<(), IpcError> {
        let index = self.active_stream_index;
        let mut streams = self
            .streams
            .lock()
            .map_err(|_| IpcError::Receive("[Unix] Unable to obtain stream lock".to_string()))?;
        let stream = streams.get_mut(index).ok_or(IpcError::Receive(
            "[Unix] No stream at active index".to_string(),
        ))?;

        let encoded = bincode::serialize(&data).map_err(|_| {
            IpcError::Serialize("[Unix] Unable to encode message data".to_string())
        })?;
        stream.write_all(&encoded).map_err(|_| {
            IpcError::Send("[Unix] Unable to write message data to stream".to_string())
        })?;
        Ok(())
    }

    pub fn receive(&self) -> Result<RendererMessage, IpcError> {
        let index = self.active_stream_index;
        let mut streams = self
            .streams
            .lock()
            .map_err(|_| IpcError::Receive("[Unix] Unable to obtain stream lock".to_string()))?;
        let stream = streams.get_mut(index).ok_or(IpcError::Receive(
            "[Unix] No stream at active index".to_string(),
        ))?;

        let mut buf = Vec::new();
        stream
            .read_to_end(&mut buf)
            .map_err(|_| IpcError::Read("[Unix] Unable to read message data".to_string()))?;

        let decoded = bincode::deserialize(&buf).map_err(|_| {
            IpcError::Deserialize("[Unix] Unable to decode message data".to_string())
        })?;

        Ok(decoded)
    }
}

impl IpcRenderer {
    pub fn new() -> Self {
        let stream = UnixStream::connect(IPC_ADDRESS).expect("Unable to create IPC renderer");

        Self {
            stream
        }
    }

    pub fn send(&self, data: RendererMessage) -> Result<(), IpcError> {
        let mut stream = self.stream.try_clone().expect("Unable to clone stream IPC renderer");
        let encoded = bincode::serialize(&data).map_err(|_| {
            IpcError::Serialize("[Unix] Unable to encode message data".to_string())
        })?;
        stream.write_all(&encoded).map_err(|_| {
            IpcError::Send("[Unix] Unable to write message data to stream".to_string())
        })?;
        Ok(())
    }

    pub fn receive(&self) -> Result<KernelMessage, IpcError> {
        let mut stream = self.stream.try_clone().expect("Unable to clone stream IPC renderer");
        let mut buf = Vec::new();
        stream
            .read_to_end(&mut buf)
            .map_err(|_| IpcError::Read("[Unix] Unable to read message data".to_string()))?;

        let decoded = bincode::deserialize(&buf).map_err(|_| {
            IpcError::Deserialize("[Unix] Unable to decode message data".to_string())
        })?;

        Ok(decoded)
    }
}