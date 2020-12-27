pub mod block;
pub mod inline;

#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContext {
    Block,
    Inline
}