use std::rc::{Rc, Weak};

use crate::element::Element;
use crate::node::{Node, NodeData};

use super::elements::*;

macro_rules! translate {
    ($tag_name:ident, {$($($matcher:pat)|* => $dataKey:ident > $result:ident),*}) => {
        match $tag_name {
            $(
                $($matcher)|* => translate!($tag_name, $dataKey, $result)
            ),*,
            _ => Node::new(NodeData::Element(Element::new(ElementData::Unknown(HTMLUnknownElement::new($tag_name.to_string())))))
        }
    };
    ($tag_name:ident, $dataKey:ident, $struct:ident) => {
        Node::new(NodeData::Element(Element::new(ElementData::$dataKey($struct::empty()))))
    }
}

pub fn create_element(document: Weak<Node>, tag_name: &str) -> Rc<Node> {
    let node = translate!(tag_name, {
        "html" => Html > HTMLHtmlElement,
        "head" => Head > HTMLHeadElement,
        "title" => Title > HTMLTitleElement,
        "body" => Body > HTMLBodyElement,
        "div" => Div > HTMLDivElement,
        "a" => Anchor > HTMLAnchorElement,
        "link" => Link > HTMLLinkElement
    });

    node.set_document(document);
    Rc::new(node)
}
