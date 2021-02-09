#[cfg(target_family = "unix")]
mod unix {
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::ops::Deref;

    pub struct Listener(UnixListener);
    pub struct Stream;

    impl Listener {
        pub fn bind() -> std::io::Result<Self> {
            let listener = UnixListener::bind("/tmp/moon/socket")?;
            Ok(Self(listener))
        }
    }

    impl Stream {
        pub fn connect() -> std::io::Result<UnixStream> {
            let stream = UnixStream::connect("/tmp/moon/socket")?;
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

#[cfg(not(target_family = "unix"))]
mod other {
    pub const PORT: u16 = 4444;
    use std::net::{TcpListener, TcpStream, SocketAddr};
    use std::ops::Deref;

    pub struct Listener(TcpListener);
    pub struct Stream;

    impl Listener {
        pub fn bind() -> std::io::Result<Self> {
            let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))?;
            Ok(Self(listener))
        }
    }

    impl Stream {
        pub fn connect() -> std::io::Result<TcpStream> {
            let stream = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], port)))?;
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

#[cfg(target_family = "unix")]
pub use unix::*;

#[cfg(not(target_family = "unix"))]
pub use other::*;
