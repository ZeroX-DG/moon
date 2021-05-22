use glium::texture::{ClientFormat, RawImage2d, Texture2dDataSource};
use std::borrow::Cow;

pub type Pixel = (u8, u8, u8, u8);

pub struct DisplayFrame {
    data: Vec<Pixel>,
    width: u32,
    height: u32
}

impl DisplayFrame {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![(255, 255, 255, 255); (width * height) as usize],
            width,
            height
        }
    }

    pub fn set_data(&mut self, data: Vec<Pixel>) {
        self.data = data;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl<'a> Texture2dDataSource<'a> for &'a DisplayFrame {
    type Data = u8;
    fn into_raw(self) -> RawImage2d<'a, Self::Data> {
        RawImage2d {
            data: Cow::Borrowed(unsafe {
                std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * 4)
            }),
            width: self.width,
            height: self.height,
            format: ClientFormat::U8U8U8U8,
        }
    }
}
