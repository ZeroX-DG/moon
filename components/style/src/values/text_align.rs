use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

impl TextAlign {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => match value {
                v if v.eq_ignore_ascii_case("left") => Some(TextAlign::Left),
                v if v.eq_ignore_ascii_case("center") => Some(TextAlign::Center),
                v if v.eq_ignore_ascii_case("right") => Some(TextAlign::Right),
                v if v.eq_ignore_ascii_case("justify") => Some(TextAlign::Justify),
                _ => None,
            },
            _ => None,
        }
    }
}
