mod unix;

#[cfg(target_os = "linux")]
pub use unix::*;
