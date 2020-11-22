/// Ipc client for kernel & renderer
///
/// Most of the code here is taken from the awesome work
/// of @unrealhoang from the project LSPC
/// https://github.com/unrealhoang/lspc/blob/master/src/rpc.rs

use std::{
    io::{BufRead, BufReader, Read, Write},
    thread,
};

use flume::bounded;

pub use flume::{Sender, Receiver, RecvError, SendError};

pub trait Message: Sized + Send + 'static {
    fn read(r: &mut impl BufRead) -> Result<Option<Self>, IpcError>;
    fn write(self, w: &mut impl Write) -> Result<(), IpcError>;
    fn is_exit(&self) -> bool;
}

#[derive(Debug)]
pub enum IpcError {
    Deserialize(String),
    Read(String),
    Write(String),
    Serialize(String),
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::Deserialize(e) => write!(f, "Deserialize Error: {}", e),
            IpcError::Serialize(e) => write!(f, "Serialize Error: {}", e),
            IpcError::Write(e) => write!(f, "Write Error: {}", e),
            IpcError::Read(e) => write!(f, "Read Error: {}", e),
        }
    }
}

impl std::convert::From<IpcError> for String {
    fn from(c: IpcError) -> Self {
        match c {
            IpcError::Deserialize(e) => format!("Deserialize Error: {}", e),
            IpcError::Serialize(e) => format!("Serialize Error: {}", e),
            IpcError::Write(e) => format!("Write Error: {}", e),
            IpcError::Read(e) => format!("Read Error: {}", e),
        }
    }
}

impl std::error::Error for IpcError {}

#[derive(Debug)]
pub struct Threads {
    reader: thread::JoinHandle<Result<(), IpcError>>,
    writer: thread::JoinHandle<Result<(), IpcError>>,
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
pub struct Client<MsgIn, MsgOut>
where
    MsgIn: Message,
    MsgOut: Message,
{
    pub sender: Sender<MsgOut>,
    pub receiver: Receiver<MsgIn>,
    threads: Threads,
}

impl<MsgIn: Message, MsgOut: Message> Client<MsgIn, MsgOut> {
    pub fn new<RF, WF, R, W>(get_reader: RF, get_writer: WF) -> Self
    where
        RF: FnOnce() -> R,
        WF: FnOnce() -> W,
        R: Read + Sized,
        W: Write + Sized,
        RF: Send + 'static,
        WF: Send + 'static,
    {
        let (writer_sender, writer_receiver) = bounded::<MsgOut>(16);
        let writer = thread::spawn(move || {
            let mut io_writer = get_writer();
            writer_receiver.into_iter().for_each(|msg| {
                if let Err(e) = msg.write(&mut io_writer) {
                    println!("Failed to write message {}", e);
                }
            });
            Ok(())
        });

        let (reader_sender, reader_receiver) = bounded::<MsgIn>(16);
        let reader = thread::spawn(move || {
            let io_reader = get_reader();
            let mut buf_read = BufReader::new(io_reader);
            loop {
                match MsgIn::read(&mut buf_read) {
                    Ok(Some(msg)) => {
                        let is_exit = msg.is_exit();

                        reader_sender.send(msg).unwrap();

                        if is_exit {
                            break;
                        }
                    }
                    Ok(None) => continue,
                    Err(e) => println!("Error reading message: {:?}", e),
                }
            }
            Ok(())
        });
        let threads = Threads { reader, writer };

        Client {
            sender: writer_sender,
            receiver: reader_receiver,
            threads,
        }
    }

    pub fn close(self) -> Result<(), String> {
        self.threads.join()
    }
}

