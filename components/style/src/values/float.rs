use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Float {
    Left,
    Right,
    None,
}

impl Float {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => {
                if value.eq_ignore_ascii_case("left") {
                    Some(Float::Left)
                } else if value.eq_ignore_ascii_case("right") {
                    Some(Float::Right)
                } else if value.eq_ignore_ascii_case("none") {
                    Some(Float::None)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
