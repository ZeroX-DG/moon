#[cfg(unix)]
mod unix {
    use std::ops::Deref;
    use std::os::unix::net::{UnixListener, UnixStream};

    const SOCKET_PATH: &str = "/tmp/moon/ipc.sock";

    pub struct Listener(UnixListener);
    pub struct Stream;

    impl Listener {
        pub fn bind() -> std::io::Result<Self> {
            if std::fs::metadata(SOCKET_PATH).is_ok() {
                // unbind old socket
                std::fs::remove_file(SOCKET_PATH)?;
            }
            let listener = UnixListener::bind(SOCKET_PATH)?;
            Ok(Self(listener))
        }
    }

    impl Stream {
        pub fn connect() -> std::io::Result<UnixStream> {
            let stream = UnixStream::connect(SOCKET_PATH)?;
            Ok(stream)
        }
    }

    impl Deref for Listener {
        type Target = UnixListener;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[cfg(not(unix))]
mod other {
    // TODO: Some how pass a port to the IPC
    // An untested solution:
    // 1. Find an unused port (from kernel)
    // 2. Bind the IPC at that port
    // 3. Spawn renderer passing the port we found
    // 4. Connect IPC renderer to IPC main
    pub const PORT: u16 = 4444;
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::ops::Deref;

    pub struct Listener(TcpListener);
    pub struct Stream;

    impl Listener {
        pub fn bind() -> std::io::Result<Self> {
            let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], PORT)))?;
            Ok(Self(listener))
        }
    }

    impl Stream {
        pub fn connect() -> std::io::Result<TcpStream> {
            let stream = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], PORT)))?;
            Ok(stream)
        }
    }

    impl Deref for Listener {
        type Target = TcpListener;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[cfg(unix)]
pub use unix::*;

#[cfg(not(unix))]
pub use other::*;
