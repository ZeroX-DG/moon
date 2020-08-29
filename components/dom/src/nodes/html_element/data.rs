use html_elements::{
    HTMLHtmlElement,
    HTMLHeadElement,
    HTMLTitleElement,
};

pub enum HTMLElementData {
    HTMLUnknownElement,
    HTMLElement,
    HTMLHtmlElement(HTMLHtmlElement),
    HTMLHeadElement(HTMLHeadElement),
    HTMLTitleElement(HTMLTitleElement),
}
