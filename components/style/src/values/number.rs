use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Number(pub f32);

impl Deref for Number {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{}", self.0).hash(state)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        self.0 == other.0
    }
}

impl Eq for Number {}

impl Into<Number> for f32 {
    fn into(self) -> Number {
        Number(self)
    }
}

impl Into<Number> for u32 {
    fn into(self) -> Number {
        Number(self as f32)
    }
}

impl Number {
    pub fn as_u8(&self) -> u8 {
        self.0 as u8
    }
}
