#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    DOCTYPE {
        name: Option<String>,
        public_identifier: Option<String>,
        system_identifier: Option<String>,
        force_quirks: bool,
    },
    Tag {
        tag_name: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
        is_end_tag: bool,
        self_closing_acknowledged: bool,
    },
    Comment(String),
    Character(char),
    EOF,
}

impl Token {
    pub fn new_start_tag() -> Self {
        Token::Tag {
            tag_name: String::new(),
            is_end_tag: false,
            self_closing: false,
            attributes: Vec::new(),
            self_closing_acknowledged: false,
        }
    }

    pub fn new_end_tag() -> Self {
        Token::Tag {
            tag_name: String::new(),
            is_end_tag: true,
            self_closing: false,
            attributes: Vec::new(),
            self_closing_acknowledged: false,
        }
    }

    pub fn new_comment(data: &str) -> Self {
        Token::Comment(data.to_owned())
    }

    pub fn new_doctype() -> Self {
        Token::DOCTYPE {
            name: None,
            public_identifier: None,
            system_identifier: None,
            force_quirks: false,
        }
    }

    pub fn set_force_quirks(&mut self, value: bool) {
        if let Token::DOCTYPE {
            ref mut force_quirks,
            ..
        } = self
        {
            *force_quirks = value;
        }
    }

    pub fn is_start_tag(&self) -> bool {
        if let Token::Tag { is_end_tag, .. } = self {
            return !*is_end_tag;
        }
        return false;
    }

    pub fn is_end_tag(&self) -> bool {
        if let Token::Tag { is_end_tag, .. } = self {
            return *is_end_tag;
        }
        return false;
    }

    pub fn tag_name(&self) -> &String {
        if let Token::Tag { tag_name, .. } = self {
            return tag_name;
        }
        panic!("Token is not a tag");
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        if let Token::Tag { attributes, .. } = self {
            return attributes;
        }
        panic!("Token is not a tag");
    }

    pub fn acknowledge_self_closing_if_set(&mut self) {
        if let Token::Tag {
            ref mut self_closing_acknowledged,
            self_closing,
            ..
        } = self
        {
            if *self_closing {
                *self_closing_acknowledged = true;
            }
        }
    }
}

impl Attribute {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
        }
    }
}
