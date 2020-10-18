use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

impl Display {
    pub fn parse(tokens: Vec<Token>) -> Option<Self> {
        Some(Display::Block)
    }
}
