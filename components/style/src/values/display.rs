use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Display {
    Inline,
    Block,
    InlineBlock,
    None,
}

impl Display {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => {
                match value {
                    v if v.eq_ignore_ascii_case("inline") => Some(Display::Inline),
                    v if v.eq_ignore_ascii_case("block") => Some(Display::Block),
                    v if v.eq_ignore_ascii_case("inline-block") => Some(Display::InlineBlock),
                    v if v.eq_ignore_ascii_case("none") => Some(Display::None),
                    _ => None
                }
            }
            _ => None,
        }
    }
}
