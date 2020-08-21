mod insert_mode;

use std::env;
use insert_mode::InsertMode;
use super::tokenizer::token::Token;
use dom::node::{NodeType, NodeRef, NodeInner};
use dom::nodes::{Document, Comment, DocumentType, QuirksMode};
use dom::implementations::Node;

fn is_trace() -> bool {
    match env::var("TRACE_TREE_BUILDER") {
        Ok(s) => s == "true",
        _ => false
    }
}

macro_rules! trace {
    ($err:expr) => {
        println!("[ParseError][TreeBuilding]: {}", $err);
    }
}

macro_rules! emit_error {
    ($err:expr) => {
        if is_trace() {
            trace!($err)
        }
    }
}

pub struct TreeBuilder {
    // stack of open elements as mentioned in specs
    open_elements: Vec<NodeRef>,

    // current insertion mode
    insert_mode: InsertMode,

    // the result document
    document: NodeRef
}

pub enum TreeBuildingStatus {
    Continue,
    Stop
}

fn is_whitespace(c: char) -> bool {
    match c {
        '\t' | '\n' | '\x0C' | ' ' => true,
        _ => false
    }
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            open_elements: Vec::new(),
            insert_mode: InsertMode::Initial,
            document: NodeRef::new_node(
                NodeType::Document,
                NodeInner::Document(Document::new())
            )
        }
    }

    pub fn process(&mut self, token: Token) -> TreeBuildingStatus {
        match self.insert_mode {
            InsertMode::Initial => self.handle_initial(token),
            _ => unimplemented!()
        }
    }

    fn which_quirks_mode(&self, token: Token) -> QuirksMode {
        if let Token::DOCTYPE {
            name,
            public_identifier,
            system_identifier,
            force_quirks
        } = token {
            // TODO: Implement full stpecs detection
            if force_quirks || name.unwrap_or_default() != "html" {
                return QuirksMode::Quirks;
            }
        }
        QuirksMode::NoQuirks
    }

    fn switch_to(&mut self, mode: InsertMode) {
        self.insert_mode = mode;
    }

    fn handle_initial(&mut self, token: Token) -> TreeBuildingStatus {
        match token.clone() {
            Token::Character(c) if is_whitespace(c) => TreeBuildingStatus::Continue,
            Token::Comment(data) => {
                let comment = NodeRef::new_node(
                    NodeType::Comment,
                    NodeInner::Comment(Comment::new(data))
                );
                self.document.append_child(comment);
                TreeBuildingStatus::Continue
            }
            Token::DOCTYPE { name, public_identifier, system_identifier, .. } => {
                let name = name.unwrap_or_default();

                let error = match (name.as_str(), public_identifier.clone(), system_identifier.clone()) {
                    ("html", None, None)                                  => false,
                    ("html", None, Some(c)) if c == "about:legacy-compat" => false,
                    _ => true
                };

                if error {
                    emit_error!("Bad doctype");
                }

                let public_id = public_identifier.unwrap_or_default();
                let system_id = system_identifier.unwrap_or_default();
                let doctype = NodeRef::new_node(
                    NodeType::DocumentType,
                    NodeInner::DocumentType(DocumentType::new(name, public_id, system_id))
                );
                

                if let NodeInner::Document(doc) = &mut *self.document.inner().borrow_mut() {
                    doc.set_doctype(Some(doctype.clone()));
                    doc.set_mode(self.which_quirks_mode(token));
                }

                self.document.append_child(doctype);

                self.switch_to(InsertMode::BeforeHtml);

                TreeBuildingStatus::Continue
            }
            _ => {
                emit_error!("Bad token");
                self.switch_to(InsertMode::BeforeHtml);
                self.process(token)
            }
        }
    }
}
