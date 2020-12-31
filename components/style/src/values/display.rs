use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Display {
    Full(OuterDisplayType, InnerDisplayType),
    Box(DisplayBox),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum OuterDisplayType {
    Block,
    Inline,
    RunIn,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum InnerDisplayType {
    Flow,
    FlowRoot,
    Table,
    Flex,
    Grid,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DisplayBox {
    Contents,
    None,
}

macro_rules! match_ident {
    ($value:expr, {$($matcher:expr => $result:expr),*}) => {
        match $value {
            ComponentValue::PerservedToken(Token::Ident(v)) => match v {
                $(
                    v if v.eq_ignore_ascii_case($matcher) => Some($result)
                ),*,
                _ => None
            }
            _ => None
        }
    };
}

impl Display {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.len() {
            1 => match_ident!(&values[0], {
                "none" => Display::Box(DisplayBox::None),
                "contents" => Display::Box(DisplayBox::Contents),
                "block" => Self::new_block(),
                "inline" => Self::new_inline()
            }),
            _ => None,
        }
    }

    pub fn new_block() -> Self {
        Display::Full(OuterDisplayType::Block, InnerDisplayType::Flow)
    }

    pub fn new_inline() -> Self {
        Display::Full(OuterDisplayType::Inline, InnerDisplayType::Flow)
    }
}
