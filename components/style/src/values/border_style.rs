use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum BorderStyle {
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
    None,
}

impl BorderStyle {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => match value {
                v if v.eq_ignore_ascii_case("hidden") => Some(BorderStyle::Hidden),
                v if v.eq_ignore_ascii_case("dotted") => Some(BorderStyle::Dotted),
                v if v.eq_ignore_ascii_case("dashed") => Some(BorderStyle::Dashed),
                v if v.eq_ignore_ascii_case("solid") => Some(BorderStyle::Solid),
                v if v.eq_ignore_ascii_case("double") => Some(BorderStyle::Double),
                v if v.eq_ignore_ascii_case("groove") => Some(BorderStyle::Groove),
                v if v.eq_ignore_ascii_case("ridge") => Some(BorderStyle::Ridge),
                v if v.eq_ignore_ascii_case("inset") => Some(BorderStyle::Inset),
                v if v.eq_ignore_ascii_case("outset") => Some(BorderStyle::Outset),
                v if v.eq_ignore_ascii_case("none") => Some(BorderStyle::None),
                _ => None,
            },
            _ => None,
        }
    }
}
