mod backend;
mod canvas;
mod painters;
mod text;
mod triangle;
mod gfx;
mod text_measure;

pub type Bitmap = Vec<u8>;

pub use canvas::Canvas;
pub use gfx::Gfx;
pub use text_measure::TextMeasure;
