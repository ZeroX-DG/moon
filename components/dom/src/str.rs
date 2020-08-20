use std::ops::Deref;

pub struct DOMString(String);
pub struct USVString(String);

impl Deref for DOMString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for USVString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
