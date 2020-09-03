mod insert_mode;
mod stack_of_open_elements;

use std::env;
use insert_mode::InsertMode;
use stack_of_open_elements::StackOfOpenElements;
use super::tokenizer::token::Token;
use dom::node::{NodeType, NodeRef, NodeInner};
use dom::nodes::{Document, Comment, DocumentType, QuirksMode};
use dom::implementations::{Node, Element};
use dom::element_factory::create_element;

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

macro_rules! match_any {
    ($target:ident, $($cmp:expr), *) => {
        $($target == $cmp)||*
    };
}

pub struct TreeBuilder {
    // stack of open elements as mentioned in specs
    open_elements: StackOfOpenElements,

    // current insertion mode
    insert_mode: InsertMode,

    // the result document
    document: NodeRef,

    // enable or disable foster parenting
    foster_parenting: bool,

    // element pointer to head element
    head_pointer: Option<NodeRef>
}

pub struct AdjustedInsertionLocation {
    pub parent: NodeRef,
    pub insert_before_sibling: Option<NodeRef>
}

pub enum ProcessingResult {
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
            open_elements: StackOfOpenElements::new(),
            insert_mode: InsertMode::Initial,
            document: NodeRef::new_node(
                NodeType::Document,
                NodeInner::Document(Document::new())
            ),
            foster_parenting: false,
            head_pointer: None
        }
    }

    pub fn process(&mut self, token: Token) -> ProcessingResult {
        match self.insert_mode {
            InsertMode::Initial => self.handle_initial(token),
            InsertMode::BeforeHtml => self.handle_before_html(token),
            InsertMode::BeforeHead => self.handle_before_head(token),
            _ => unimplemented!()
        }
    }

    fn which_quirks_mode(&self, token: Token) -> QuirksMode {
        if let Token::DOCTYPE {
            name,
            force_quirks,
            ..
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

    fn create_element(&self, tag_token: Token) -> NodeRef {
        let (tag_name, attributes) = if let Token::Tag { tag_name, attributes, .. } = tag_token {
            (tag_name, attributes)
        } else {
            ("".to_string(), Vec::new())
        };
        let mut element = create_element(self.document.clone().downgrade(), &tag_name);
        for attribute in attributes {
            element.set_attribute(&attribute.name, &attribute.value);
        }
        element
    }

    fn create_element_from_tag_name(&self, tag_name: &str) -> NodeRef {
        self.create_element(Token::Tag {
            tag_name: tag_name.to_owned(),
            self_closing: false,
            attributes: Vec::new(),
            is_end_tag: false
        })
    }

    fn get_appropriate_place_for_inserting_a_node(&self) -> AdjustedInsertionLocation {
        let target = self.open_elements.current_node().unwrap();
        
        // TODO: implement full specs
        return AdjustedInsertionLocation {
            parent: target.clone(),
            insert_before_sibling: target.last_child()
        };
    }

    fn insert_html_element(&mut self, token: Token) -> NodeRef {
        let insert_position = self.get_appropriate_place_for_inserting_a_node();
        let element = self.create_element(token);
        let return_ref = element.clone();
        
        // TODO: check if location is possible to insert node (Idk why so we just leave it for now)
        self.open_elements.push(element.clone());
        insert_position.parent.insert_before(element, insert_position.insert_before_sibling);
        return_ref
    }
    
}

impl TreeBuilder {
    fn handle_initial(&mut self, token: Token) -> ProcessingResult {
        match token.clone() {
            Token::Character(c) if is_whitespace(c) => ProcessingResult::Continue,
            Token::Comment(data) => {
                let comment = NodeRef::new_node(
                    NodeType::Comment,
                    NodeInner::Comment(Comment::new(data))
                );
                self.document.append_child(comment);
                ProcessingResult::Continue
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

                ProcessingResult::Continue
            }
            _ => {
                emit_error!("Bad token");
                self.switch_to(InsertMode::BeforeHtml);
                self.process(token)
            }
        }
    }

    fn handle_before_html(&mut self, token: Token) -> ProcessingResult {
        fn anything_else(this: &mut TreeBuilder, token: Token) -> ProcessingResult {
            let element = this.create_element_from_tag_name("html");
            this.document.append_child(element.clone());
            this.open_elements.push(element.clone());
            // TODO: Implement additional steps in specs
            this.switch_to(InsertMode::BeforeHead);
            this.process(token.clone())
        }

        match token.clone() {
            Token::DOCTYPE { .. } => {
                emit_error!("Unexpected doctype");
                ProcessingResult::Continue
            }
            Token::Comment(data) => {
                let comment = NodeRef::new_node(
                    NodeType::Comment,
                    NodeInner::Comment(Comment::new(data))
                );
                self.document.append_child(comment);
                ProcessingResult::Continue
            }
            Token::Character(c) if is_whitespace(c) => ProcessingResult::Continue,
            Token::Tag { tag_name, is_end_tag, .. } => {
                if tag_name == "html" && !is_end_tag {
                    let element = self.create_element(token);
                    self.document.append_child(element.clone());
                    self.open_elements.push(element.clone());
                    // TODO: implement additional steps in specs
                    self.switch_to(InsertMode::BeforeHead);
                    ProcessingResult::Continue
                } else if match_any!(tag_name, "head", "body", "html", "br") && is_end_tag {
                    anything_else(self, token)
                } else {
                    if is_end_tag {
                        emit_error!("Invalid end tag");
                        return ProcessingResult::Continue;
                    }
                    anything_else(self, token)
                }
            }
            _ => anything_else(self, token)
        }
    }

    fn handle_before_head(&mut self, token: Token) -> ProcessingResult {
        fn anything_else(this: &mut TreeBuilder, token: Token) -> ProcessingResult {
            let head_element = this.insert_html_element(Token::Tag {
                tag_name: "head".to_owned(),
                attributes: Vec::new(),
                is_end_tag: false,
                self_closing: false
            });
            this.head_pointer = Some(head_element.clone());
            this.switch_to(InsertMode::InHead);
            this.process(token)
        }
        match token.clone() {
            Token::Character(c) if is_whitespace(c) => ProcessingResult::Continue,
            Token::Comment(data) => {
                let insert_position = self.get_appropriate_place_for_inserting_a_node();
                let comment = NodeRef::new_node(
                    NodeType::Comment,
                    NodeInner::Comment(Comment::new(data))
                );
                insert_position.parent.insert_before(comment, insert_position.insert_before_sibling);
                ProcessingResult::Continue
            }
            Token::DOCTYPE { .. } => {
                emit_error!("Unexpected doctype");
                ProcessingResult::Continue
            }
            Token::Tag { tag_name, is_end_tag, .. } => {
                if !is_end_tag && tag_name == "html" {
                    return self.handle_in_body(token);
                }
                if !is_end_tag && tag_name == "head" {
                    let head_element = self.insert_html_element(token);
                    self.head_pointer = Some(head_element);
                    self.switch_to(InsertMode::InHead);
                    return ProcessingResult::Continue
                }
                if is_end_tag && match_any!(tag_name, "head", "body", "html", "br") {
                    return anything_else(self, token);
                }
                if is_end_tag {
                    emit_error!("Invalid end tag");
                    return ProcessingResult::Continue
                }
                anything_else(self, token)
            }
            _ => anything_else(self, token)
        }
    }

    fn handle_in_body(&mut self, token: Token) -> ProcessingResult {
        ProcessingResult::Continue
    }
}
