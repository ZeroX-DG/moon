use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Number(f32);

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
