mod backend;
mod canvas;
mod painters;
mod text;
mod triangle;
mod gfx;

pub type Bitmap = Vec<u8>;

pub use canvas::Canvas;
pub use gfx::Gfx;
