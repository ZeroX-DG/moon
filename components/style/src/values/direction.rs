use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    Ltr,
    Rtl,
}

impl Direction {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => {
                if value.eq_ignore_ascii_case("ltr") {
                    Some(Direction::Ltr)
                } else if value.eq_ignore_ascii_case("rtl") {
                    Some(Direction::Rtl)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
