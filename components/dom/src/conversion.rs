pub trait HTMLElement {}

pub trait HTMLElementConvert<T: HTMLElement> {
    fn as_element(&self) -> T;
}

#[macro_export]
macro_rules! impl_html_convert {
    ($element:tt) => {
        use crate::conversion::*;
        impl HTMLElement for $element {}
        impl HTMLElementConvert<$element> for NodeRef {
            fn as_element(&self) -> $element {
                $element::new(self.clone())
            }
        }
    };
}
