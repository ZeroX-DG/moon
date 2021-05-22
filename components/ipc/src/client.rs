use std::{
    io::{BufRead, BufReader, Read, Write},
    thread,
};

use flume::bounded;

pub use flume::{Receiver, RecvError, Selector, SendError, Sender};

pub trait Message: Sized + Send + 'static {
    fn read(r: &mut impl BufRead) -> Result<Option<Self>, IpcTransportError>;
    fn write(self, w: &mut impl Write) -> Result<(), IpcTransportError>;
    fn is_exit(&self) -> bool;
}

#[derive(Debug)]
pub enum IpcTransportError {
    Deserialize(String),
    Read(String),
    Write(String),
    Serialize(String),
}

impl std::fmt::Display for IpcTransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcTransportError::Deserialize(e) => write!(f, "Deserialize Error: {}", e),
            IpcTransportError::Serialize(e) => write!(f, "Serialize Error: {}", e),
            IpcTransportError::Write(e) => write!(f, "Write Error: {}", e),
            IpcTransportError::Read(e) => write!(f, "Read Error: {}", e),
        }
    }
}

impl std::convert::From<IpcTransportError> for String {
    fn from(c: IpcTransportError) -> Self {
        match c {
            IpcTransportError::Deserialize(e) => format!("Deserialize Error: {}", e),
            IpcTransportError::Serialize(e) => format!("Serialize Error: {}", e),
            IpcTransportError::Write(e) => format!("Write Error: {}", e),
            IpcTransportError::Read(e) => format!("Read Error: {}", e),
        }
    }
}

impl std::error::Error for IpcTransportError {}

#[derive(Debug)]
pub struct Threads {
    reader: thread::JoinHandle<Result<(), IpcTransportError>>,
    writer: thread::JoinHandle<Result<(), IpcTransportError>>,
}

impl Threads {
    pub fn join(self) -> Result<(), String> {
        match self.reader.join() {
            Ok(r) => r?,
            Err(_) => Err("reader panicked")?,
        };
        match self.writer.join() {
            Ok(r) => r?,
            Err(_) => Err("writer panicked")?,
        };
        Ok(())
    }
}

#[derive(Debug)]
pub struct Client<M: Message> {
    pub id: String,
    pub sender: Sender<M>,
    pub receiver: Receiver<M>,
    threads: Threads,
}

impl<M: Message> Client<M> {
    pub fn new<RF, WF, R, W>(get_reader: RF, get_writer: WF) -> Self
    where
        RF: FnOnce() -> R,
        WF: FnOnce() -> W,
        R: Read + Sized,
        W: Write + Sized,
        RF: Send + 'static,
        WF: Send + 'static,
    {
        let (writer_sender, writer_receiver) = bounded::<M>(16);
        let writer = thread::spawn(move || {
            let mut io_writer = get_writer();
            writer_receiver.into_iter().for_each(|msg| {
                if let Err(e) = msg.write(&mut io_writer) {
                    log::error!("Failed to write message {}", e);
                }
            });
            Ok(())
        });

        let (reader_sender, reader_receiver) = bounded::<M>(16);
        let reader = thread::spawn(move || {
            let io_reader = get_reader();
            let mut buf_read = BufReader::new(io_reader);
            loop {
                match M::read(&mut buf_read) {
                    Ok(Some(msg)) => {
                        let is_exit = msg.is_exit();

                        reader_sender.send(msg).unwrap();

                        if is_exit {
                            break;
                        }
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        log::error!("Error while reading: {:#?}", e.to_string());
                        return Err(IpcTransportError::Read("Error reading".to_string()));
                    }
                }
            }
            Ok(())
        });
        let threads = Threads { reader, writer };

        Client {
            id: nanoid::nanoid!(),
            sender: writer_sender,
            receiver: reader_receiver,
            threads,
        }
    }

    pub fn receiver(&self) -> &Receiver<M> {
        &self.receiver
    }

    pub fn sender(&self) -> &Sender<M> {
        &self.sender
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn close(self) -> Result<(), String> {
        self.threads.join()
    }
}
