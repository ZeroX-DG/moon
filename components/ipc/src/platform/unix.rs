use crate::IpcError;
use bincode;
use serde::{de::DeserializeOwned, Serialize};
use std::io::Read;
use std::marker::PhantomData;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;

const IPC_ADDRESS: &str = "/tmp/moon-ipc";

pub struct IpcMain<T>
where
    T: Serialize + DeserializeOwned,
{
    streams: Arc<Mutex<Vec<UnixStream>>>,
    active_stream_index: usize,
    generic: PhantomData<T>,
}

pub struct IpcRenderer<T>
where
    T: Serialize + DeserializeOwned,
{
    stream: UnixStream,
    gereic: PhantomData<T>
}

impl<T: Serialize + DeserializeOwned> IpcMain<T> {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(Mutex::new(Vec::new())),
            active_stream_index: 0,
            generic: PhantomData::default(),
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

    pub fn receive(&self) -> Result<T, IpcError> {
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

impl<T: Serialize + DeserializeOwned> IpcRenderer<T> {
    pub fn new() -> Self {
        let stream = UnixStream::connect(IPC_ADDRESS).expect("Unable to create IPC renderer");

        Self {
            stream,
            generic: PhantomData::default()
        }
    }
}