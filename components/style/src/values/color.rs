use super::number::Number;
use css::parser::structs::ComponentValue;
use css::parser::structs::Function;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Color {
    CurrentColor,
    Rgba(Number, Number, Number, Number),
}

impl Eq for Color {}

macro_rules! match_keyword {
    ($keyword:ident, { $($match_key:expr => $value:expr),* }) => {
        match $keyword {
            $(n if n.trim().eq_ignore_ascii_case($match_key) => Some($value)),*,
            _ => None
        }
    };
}

impl From<&Color> for shared::color::Color {
    fn from(color: &Color) -> Self {
        let default_color = shared::color::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0
        };
        match color {
            Color::Rgba(r, g, b, a) => {
                let alpha: u8 = a.as_u8();
                shared::color::Color {
                    r: r.as_u8(),
                    g: g.as_u8(),
                    b: b.as_u8(),
                    a: alpha,
                }
            }
            _ => default_color,
        } 
    }
}

impl Color {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::Function(function)) => match function.name.as_ref() {
                "rgba" => Color::parse_rgba_function(function, true),
                "rgb" => Color::parse_rgba_function(function, false),
                _ => None,
            },
            Some(ComponentValue::PerservedToken(Token::Ident(keyword))) => {
                Color::parse_color_keyword(&keyword)
            }
            Some(ComponentValue::PerservedToken(Token::Hash(data, _))) => Color::parse_hex(&data),
            _ => None,
        }
    }

    fn parse_hex(hex: &str) -> Option<Self> {
        let mut chars = hex.chars();

        fn parse_digit(chars: &mut std::str::Chars) -> Option<u32> {
            if let Some(ch) = chars.next() {
                return ch.to_digit(16);
            }
            None
        }

        fn parse_pair_digit(chars: &mut std::str::Chars) -> Option<u32> {
            let pair: String = chars.take(2).collect();

            if pair.len() == 2 {
                return u32::from_str_radix(&pair, 16).ok();
            }

            None
        }

        if hex.len() == 3 {
            let r = match parse_digit(&mut chars) {
                Some(d) => d * 0x11,
                _ => return None,
            };
            let g = match parse_digit(&mut chars) {
                Some(d) => d * 0x11,
                _ => return None,
            };
            let b = match parse_digit(&mut chars) {
                Some(d) => d * 0x11,
                _ => return None,
            };
            Some(Color::Rgba(r.into(), g.into(), b.into(), 255.0.into()))
        } else if hex.len() == 6 {
            let r = match parse_pair_digit(&mut chars) {
                Some(d) => d,
                _ => return None,
            };
            let g = match parse_pair_digit(&mut chars) {
                Some(d) => d,
                _ => return None,
            };
            let b = match parse_pair_digit(&mut chars) {
                Some(d) => d,
                _ => return None,
            };
            Some(Color::Rgba(r.into(), g.into(), b.into(), 255.0.into()))
        } else {
            None
        }
    }

    fn parse_color_keyword(keyword: &str) -> Option<Self> {
        match_keyword!(keyword, {
            "currentColor" => Color::CurrentColor,
            "transparent" => Color::transparent(),
            "black" => Color::black(),
            "silver" => Color::Rgba(
                192.0.into(),
                192.0.into(),
                192.0.into(),
                255.0.into(),
            ),
            "gray" => Color::Rgba(
                128.0.into(),
                128.0.into(),
                128.0.into(),
                255.0.into(),
            ),
            "white" => Color::Rgba(
                255.0.into(),
                255.0.into(),
                255.0.into(),
                255.0.into(),
            ),
            "maroon" => Color::Rgba(
                128.0.into(),
                0.0.into(),
                0.0.into(),
                255.0.into(),
            ),
            "red" => Color::Rgba(
                255.0.into(),
                0.0.into(),
                0.0.into(),
                255.0.into(),
            ),
            "purple" => Color::Rgba(
                128.0.into(),
                0.0.into(),
                128.0.into(),
                255.0.into(),
            ),
            "fuchsia" => Color::Rgba(
                255.0.into(),
                0.0.into(),
                255.0.into(),
                255.0.into(),
            ),
            "green" => Color::Rgba(
                0.0.into(),
                128.0.into(),
                0.0.into(),
                255.0.into(),
            ),
            "lime" => Color::Rgba(
                0.0.into(),
                255.0.into(),
                0.0.into(),
                255.0.into(),
            ),
            "olive" => Color::Rgba(
                128.0.into(),
                128.0.into(),
                0.0.into(),
                255.0.into(),
            ),
            "yellow" => Color::Rgba(
                255.0.into(),
                255.0.into(),
                0.0.into(),
                255.0.into(),
            ),
            "navy" => Color::Rgba(
                0.0.into(),
                0.0.into(),
                128.0.into(),
                255.0.into(),
            ),
            "blue" => Color::Rgba(
                0.0.into(),
                0.0.into(),
                255.0.into(),
                255.0.into(),
            ),
            "teal" => Color::Rgba(
                0.0.into(),
                128.0.into(),
                128.0.into(),
                255.0.into(),
            ),
            "aqua" => Color::Rgba(
                0.0.into(),
                255.0.into(),
                255.0.into(),
                255.0.into(),
            )
        })
    }

    fn parse_rgba_function(function: &Function, with_alpha: bool) -> Option<Self> {
        let mut rgba: [f32; 4] = if !with_alpha {
            [0.0, 0.0, 0.0, 255.0]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };

        let mut index = 0;
        let max_length = if !with_alpha { 3 } else { 4 };

        for value in &function.value {
            match value {
                ComponentValue::PerservedToken(Token::Number { value, .. }) => {
                    if index == max_length {
                        return None;
                    }
                    rgba[index] = *value;
                    index += 1;
                }
                ComponentValue::PerservedToken(Token::Whitespace) => {}
                ComponentValue::PerservedToken(Token::Comma) => {}
                _ => return None, // invalid character
            }
        }

        Some(Color::Rgba(
            rgba[0].into(),
            rgba[1].into(),
            rgba[2].into(),
            rgba[3].into(),
        ))
    }

    pub fn transparent() -> Self {
        Color::Rgba(0.0.into(), 0.0.into(), 0.0.into(), 0.0.into())
    }

    pub fn black() -> Self {
        Color::Rgba(0.0.into(), 0.0.into(), 0.0.into(), 255.0.into())
    }
}
