use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Overflow {
    Visible,
    Hidden,
    Clip,
    Scroll
}

impl Eq for Overflow {}

impl Overflow {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => match value {
                v if v.eq_ignore_ascii_case("visible") => Some(Overflow::Visible),
                v if v.eq_ignore_ascii_case("hidden") => Some(Overflow::Hidden),
                v if v.eq_ignore_ascii_case("clip") => Some(Overflow::Clip),
                v if v.eq_ignore_ascii_case("scroll") => Some(Overflow::Scroll),
                _ => None,
            },
            _ => None,
        }
    }
}
