use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum BorderWidth {
    Thin,
    Medium,
    Thick,
}

impl BorderWidth {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => match value {
                v if v.eq_ignore_ascii_case("thin") => Some(BorderWidth::Thin),
                v if v.eq_ignore_ascii_case("medium") => Some(BorderWidth::Medium),
                v if v.eq_ignore_ascii_case("thick") => Some(BorderWidth::Thick),
                _ => None,
            },
            _ => None,
        }
    }
}
