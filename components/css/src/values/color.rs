pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

pub enum ParseColorError {
    InvalidHexValue,
    BadColorName
}

impl Color {
    pub fn red(&self) -> u8 {
        self.r
    }

    pub fn green(&self) -> u8 {
        self.g
    }

    pub fn blue(&self) -> u8 {
        self.b
    }

    pub fn alpha(&self) -> u8 {
        self.a
    }
}

impl Color {
    fn from_hex(value: &str) -> Self {
        let mut hex_bytes = value.as_bytes().iter().filter_map(|b| {
            match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            }
        }).fuse();

        let mut bytes: [u8; 3] = [0, 0, 0];
        let mut index = 0;
        while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
            bytes[index] = h << 4 | l;
            index += 1;
        }

        Self {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
            a: 255
        }
    }

    pub fn parse(value: &str) -> Result<Self, ParseColorError> {
        if value.starts_with('#') {
            // skip '#'
            let color_value: &str = &value[1..];
            let seq_len = color_value.len();

            if seq_len == 3 || seq_len == 4 || seq_len == 6 {
                return Ok(Color::from_hex(color_value));
            } else {
                return Err(ParseColorError::InvalidHexValue);
            }
        }

        Err(ParseColorError::BadColorName)
    }
}
