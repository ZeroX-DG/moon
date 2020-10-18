use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

impl Display {
    pub fn parse(values: &Vec<ComponentValue>) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => {
                if value.eq_ignore_ascii_case("inline") {
                    Some(Display::Inline)
                } else if value.eq_ignore_ascii_case("block") {
                    Some(Display::Block)
                } else if value.eq_ignore_ascii_case("none") {
                    Some(Display::None)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
