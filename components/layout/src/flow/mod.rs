pub mod block;

#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContext {
    Block,
    Inline
}