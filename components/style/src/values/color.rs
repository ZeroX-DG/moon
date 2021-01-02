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
            _ => None,
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
