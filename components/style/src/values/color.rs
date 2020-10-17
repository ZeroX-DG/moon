use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Color {
    RGBA(u8, u8, u8, u8)
}

impl Color {
    pub fn parse(tokens: Vec<Token>) -> Option<Self> {
        Some(Color::RGBA(255, 255, 255, 255))
    }

    pub fn transparent() -> Self {
        Color::RGBA(0, 0, 0, 0)
    }

    pub fn black() -> Self {
        Color::RGBA(0, 0, 0, 1)
    }
}
