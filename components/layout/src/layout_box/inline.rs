use super::BaseBox;

#[derive(Debug)]
pub struct InlineBox {
    base: BaseBox
}

#[derive(Debug)]
pub struct AtomicInlineLevelBox {
    base: BaseBox
}

#[derive(Debug)]
pub struct AnonymousInlineBox {
    base: BaseBox
}
