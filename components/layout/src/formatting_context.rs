/// Formatting context of each box
#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContext {
    Inline,
    Block,
    Flex,
    Grid
}