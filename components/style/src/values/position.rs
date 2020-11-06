use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
}

impl Position {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => match value {
                v if v.eq_ignore_ascii_case("static") => Some(Position::Static),
                v if v.eq_ignore_ascii_case("relative") => Some(Position::Relative),
                v if v.eq_ignore_ascii_case("absolute") => Some(Position::Absolute),
                v if v.eq_ignore_ascii_case("fixed") => Some(Position::Fixed),
                _ => None,
            },
            _ => None,
        }
    }
}
