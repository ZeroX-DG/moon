use dom::dom_ref::{NodeRef, WeakNodeRef};
use dom::element::Element;

use super::elements::HTMLBaseElement;
use super::elements::HTMLBodyElement;
use super::elements::HTMLDivElement;
use super::elements::HTMLElement;
use super::elements::HTMLHeadElement;
use super::elements::HTMLHtmlElement;
use super::elements::HTMLScriptElement;
use super::elements::HTMLTitleElement;

macro_rules! translate {
    ($tag_name:ident, $element:ident, {$($($matcher:pat)|* => $result:ident),*}) => {
        match $tag_name {
            $(
                $($matcher)|* => translate!($tag_name, $element, $result)
            ),*,
            _ => NodeRef::new($element)
        }
    };
    ($tag_name:ident, $element:ident, HTMLElement) => {
        NodeRef::new(HTMLElement::new($element))
    };
    ($tag_name:ident, $element:ident, $struct:ident) => {
        NodeRef::new($struct::new(HTMLElement::new($element)))
    }
}

pub fn create_element(document: WeakNodeRef, tag_name: &str) -> NodeRef {
    let mut element = Element::new(tag_name.to_owned());
    element.node.set_document(document);

    translate!(tag_name, element, {
        "html" => HTMLHtmlElement,
        "head" => HTMLHeadElement,
        "base" => HTMLBaseElement,
        "title" => HTMLTitleElement,
        "body" => HTMLBodyElement,
        "div" => HTMLDivElement,
        "script" => HTMLScriptElement,

        "article"
        | "section" | "nav" | "aside" | "hgroup" | "header" | "footer"
        | "address" | "dt" | "dd" | "figure" | "figcaption" | "main"
        | "em" | "strong" | "small" | "s" | "cite" | "dfn" | "abbr"
        | "ruby" | "rt" | "rp" | "code" | "var" | "samp" | "kbd"
        | "sub" | "sup" | "i" | "b" | "u" | "mark" | "bdi" | "bdo"
        | "wbr" | "summary" | "noscript"
        => HTMLElement,

        // Obsolete
        "acronym" | "basefont" | "big" | "center" | "nobr" | "noembed"
        | "noframes" | "plaintext" | "rb" | "rtc" | "strike" | "tt"
        => HTMLElement
    })
}
