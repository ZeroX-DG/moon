use crate::html_elements::{
    HTMLHtmlElement,
    HTMLHeadElement,
    HTMLTitleElement,
    HTMLBodyElement,
    HTMLDivElement
};

pub enum HTMLElementData {
    HTMLUnknownElement,
    HTMLElement,
    HTMLHtmlElement(HTMLHtmlElement),
    HTMLHeadElement(HTMLHeadElement),
    HTMLTitleElement(HTMLTitleElement),
    HTMLBodyElement(HTMLBodyElement),
    HTMLDivElement(HTMLDivElement),
}
