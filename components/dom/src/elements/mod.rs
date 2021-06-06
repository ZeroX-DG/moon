mod html_anchor_element;
mod html_body_element;
mod html_div_element;
mod html_head_element;
mod html_html_element;
mod html_title_element;

pub use html_anchor_element::*;
pub use html_body_element::*;
pub use html_div_element::*;
pub use html_head_element::*;
pub use html_html_element::*;
pub use html_title_element::*;

#[derive(Debug)]
pub enum ElementData {
    Anchor(HTMLAnchorElement),
    Body(HTMLBodyElement),
    Div(HTMLDivElement),
    Head(HTMLHeadElement),
    Html(HTMLHtmlElement),
    Title(HTMLTitleElement),
}

