mod insert_mode;
mod list_of_active_formatting_elements;
mod open_element_types;
mod stack_of_open_elements;

use super::tokenizer::state::State;
use super::tokenizer::token::Attribute;
use super::tokenizer::token::Token;
use crate::tokenizer::Tokenizing;
use dom::comment::Comment;
use dom::document::{Document, DocumentType, QuirksMode};
use dom::element::Element;
use dom::node::ChildrenUpdateContext;
use dom::node::NodePtr;
use dom::node::{Node, NodeData};
use dom::text::Text;
use insert_mode::InsertMode;
use list_of_active_formatting_elements::Entry;
use list_of_active_formatting_elements::ListOfActiveFormattingElements;
use open_element_types::is_special_element;
use phf::phf_map;
use shared::tree_node::TreeNode;
use shared::tree_node::WeakTreeNode;
use stack_of_open_elements::StackOfOpenElements;
use std::env;
use std::rc::Rc;

fn is_trace() -> bool {
    match env::var("TRACE_HTML_TREE_BUILDER") {
        Ok(s) => s == "true",
        _ => false,
    }
}

macro_rules! trace {
    ($err:expr) => {
        println!("[ParseError][HTML TreeBuilding]: {}", $err)
    };
}

macro_rules! emit_error {
    ($err:expr) => {
        if is_trace() {
            trace!($err)
        }
    };
}

macro_rules! match_any {
    ($target:ident, $($cmp:expr), *) => {
        $($target == $cmp)||*
    };
    ($target:expr, $($cmp:expr), *) => {
        $($target == $cmp)||*
    };
}

/// The DOM tree builder
pub struct TreeBuilder<T: Tokenizing> {
    /// The tokenizer controlled by TreeBuilder
    tokenizer: T,

    /// Stack of open elements as mentioned in specs
    open_elements: StackOfOpenElements,

    /// Indicate if the tree builder should stop parsing
    should_stop: bool,

    /// Current insertion mode
    insert_mode: InsertMode,

    /// The insert mode that the builder will return
    original_insert_mode: Option<InsertMode>,

    /// The result document
    document: NodePtr,

    /// Enable or disable foster parenting
    foster_parenting: bool,

    /// Element pointer to head element
    head_pointer: Option<NodePtr>,

    /// Element pointer to last open form element
    form_pointer: Option<NodePtr>,

    /// Is scripting enable?
    scripting: bool,

    /// List of active formatting elements
    active_formatting_elements: ListOfActiveFormattingElements,

    /// `frameset_ok` flag
    frameset_ok: bool,

    /// Stack of template insert mode to parse nested template
    stack_of_template_insert_mode: Vec<InsertMode>,

    /// Pending table character tokens
    table_character_tokens: Vec<Token>,

    /// Is created to parse fragment html
    is_fragment_case: bool,

    /// Context element for fragment html
    context_element: Option<NodePtr>,

    /// Current node for text insertion
    text_insertion_node: Option<NodePtr>,

    /// Current string that hold the text for the text node data
    text_insertion_string_data: String,
}

/// The adjusted location to insert a node as mentioned the specs
pub enum AdjustedInsertionLocation {
    LastChild(NodePtr),
    BeforeSibling(NodePtr, NodePtr),
}

impl AdjustedInsertionLocation {
    pub fn parent(&self) -> &NodePtr {
        match self {
            AdjustedInsertionLocation::LastChild(parent) => parent,
            AdjustedInsertionLocation::BeforeSibling(parent, _) => parent,
        }
    }
}

/// The parsing algorithm to be used for parsing text-only element
enum TextOnlyElementParsingAlgo {
    GenericRawText,
    GenericRCDataElement,
}

enum AdoptionAgencyOutcome {
    DoNothing,
    RunAnyOtherEndTags,
}

/// Check if the character is a whitespace character according to specs
fn is_whitespace(c: char) -> bool {
    match c {
        '\t' | '\n' | '\x0C' | ' ' => true,
        _ => false,
    }
}

static SVG_ATTRIBUTE_MAP: phf::Map<&str, &str> = phf_map! {
    "attributename" => "attributeName",
    "attributetype" => "attributeType",
    "basefrequency" => "baseFrequency",
    "baseprofile" => "baseProfile",
    "calcmode" => "calcMode",
    "clippathunits" => "clipPathUnits",
    "diffuseconstant" => "diffuseConstant",
    "edgemode" => "edgeMode",
    "filterunits" => "filterUnits",
    "glyphref" => "glyphRef",
    "gradienttransform" => "gradientTransform",
    "gradientunits" => "gradientUnits",
    "kernelmatrix" => "kernelMatrix",
    "kernelunitlength" => "kernelUnitLength",
    "keypoints" => "keyPoints",
    "keysplines" => "keySplines",
    "keytimes" => "keyTimes",
    "lengthadjust" => "lengthAdjust",
    "limitingconeangle" => "limitingConeAngle",
    "markerheight" => "markerHeight",
    "markerunits" => "markerUnits",
    "markerwidth" => "markerWidth",
    "maskcontentunits" => "maskContentUnits",
    "maskunits" => "maskUnits",
    "numoctaves" => "numOctaves",
    "pathlength" => "pathLength",
    "patterncontentunits" => "patternContentUnits",
    "patterntransform" => "patternTransform",
    "patternunits" => "patternUnits",
    "pointsatx" => "pointsAtX",
    "pointsaty" => "pointsAtY",
    "pointsatz" => "pointsAtZ",
    "preservealpha" => "preserveAlpha",
    "preserveaspectratio" => "preserveAspectRatio",
    "primitiveunits" => "primitiveUnits",
    "refx" => "refX",
    "refy" => "refY",
    "repeatcount" => "repeatCount",
    "repeatdur" => "repeatDur",
    "requiredextensions" => "requiredExtensions",
    "requiredfeatures" => "requiredFeatures",
    "specularconstant" => "specularConstant",
    "specularexponent" => "specularExponent",
    "spreadmethod" => "spreadMethod",
    "startoffset" => "startOffset",
    "stddeviation" => "stdDeviation",
    "stitchtiles" => "stitchTiles",
    "surfacescale" => "surfaceScale",
    "systemlanguage" => "systemLanguage",
    "tablevalues" => "tableValues",
    "targetx" => "targetX",
    "targety" => "targetY",
    "textlength" => "textLength",
    "viewbox" => "viewBox",
    "viewtarget" => "viewTarget",
    "xchannelselector" => "xChannelSelector",
    "ychannelselector" => "yChannelSelector",
    "zoomandpan" => "zoomAndPan",
};

fn adjust_svg_attributes(token: &mut Token) {
    for attr in token.attributes_mut() {
        if let Some(rname) = SVG_ATTRIBUTE_MAP.get(attr.name.as_str()) {
            attr.name = rname.to_string();
        }
    }
}

const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";
const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

static FOREIGN_ATTRIBUTE_MAP: phf::Map<&str, (&str, &str, &str)> = phf_map! {
    "xlink:actuate" => ("xlink", "actuate", XLINK_NAMESPACE),
    "xlink:arcrole" => ("xlink", "arcrole", XLINK_NAMESPACE),
    "xlink:href" => ("xlink", "href", XLINK_NAMESPACE),
    "xlink:role" => ("xlink", "role", XLINK_NAMESPACE),
    "xlink:show" => ("xlink", "show", XLINK_NAMESPACE),
    "xlink:title" => ("xlink", "title", XLINK_NAMESPACE),
    "xlink:type" => ("xlink", "type", XLINK_NAMESPACE),
    "xml:lang" => ("xml", "lang", XML_NAMESPACE),
    "xml:space" => ("xml", "space", XML_NAMESPACE),
    "xmlns" => ("", "xmlns", XMLNS_NAMESPACE),
    "xmlns:xlink" => ("xmlns", "xlink", XMLNS_NAMESPACE),
};

fn adjust_foreign_attributes(token: &mut Token) {
    for attr in token.attributes_mut() {
        if let Some(data) = FOREIGN_ATTRIBUTE_MAP.get(attr.name.as_str()) {
            let (prefix, name, namespace) = data;
            attr.name = name.to_string();
            attr.prefix = prefix.to_string();
            attr.namespace = namespace.to_string();
        }
    }
}

impl<T: Tokenizing> TreeBuilder<T> {
    pub fn new(tokenizer: T, document: NodePtr) -> Self {
        Self {
            tokenizer,
            open_elements: StackOfOpenElements::new(),
            insert_mode: InsertMode::Initial,
            document,
            foster_parenting: false,
            head_pointer: None,
            form_pointer: None,
            original_insert_mode: None,
            scripting: false,
            active_formatting_elements: ListOfActiveFormattingElements::new(),
            frameset_ok: true,
            stack_of_template_insert_mode: Vec::new(),
            should_stop: false,
            table_character_tokens: Vec::new(),
            is_fragment_case: false,
            context_element: None,
            text_insertion_node: None,
            text_insertion_string_data: String::new(),
        }
    }

    /// Create a HTML tree builder with default document & no loader.
    /// This should only be used for testing
    pub fn default(tokenizer: T) -> Self {
        let document = NodePtr(TreeNode::new(Node::new(NodeData::Document(Document::new()))));
        Self::new(tokenizer, document)
    }

    /// Start the main loop for parsing DOM tree
    pub fn run(mut self) -> NodePtr {
        loop {
            let token = self.tokenizer.next_token();

            self.process(token);

            if self.should_stop {
                break;
            }
        }
        self.flush_text_insertion();
        return self.document;
    }

    /// (Re)process a token in the current insert mode
    pub fn process(&mut self, token: Token) {
        match self.insert_mode {
            InsertMode::Initial => self.handle_initial(token),
            InsertMode::BeforeHtml => self.handle_before_html(token),
            InsertMode::BeforeHead => self.handle_before_head(token),
            InsertMode::InHead => self.handle_in_head(token),
            InsertMode::InHeadNoScript => self.handle_in_head_no_script(token),
            InsertMode::AfterHead => self.handle_after_head(token),
            InsertMode::InBody => self.handle_in_body(token),
            InsertMode::Text => self.handle_text(token),
            InsertMode::InTable => self.handle_in_table(token),
            InsertMode::InTableText => self.handle_in_table_text(token),
            InsertMode::InCaption => self.handle_in_caption(token),
            InsertMode::InColumnGroup => self.handle_in_column_group(token),
            InsertMode::InTableBody => self.handle_in_table_body(token),
            InsertMode::InRow => self.handle_in_row(token),
            InsertMode::InCell => self.handle_in_cell(token),
            InsertMode::InSelect => self.handle_in_select(token),
            InsertMode::InSelectInTable => self.handle_in_select_in_table(token),
            InsertMode::AfterBody => self.handle_after_body(token),
            InsertMode::AfterAfterBody => self.handle_after_after_body(token),
            _ => unimplemented!(),
        }
    }

    /// Get the current parsing document
    pub fn get_document(&self) -> NodePtr {
        self.document.clone()
    }

    fn which_quirks_mode(&self, token: Token) -> QuirksMode {
        if let Token::DOCTYPE {
            name, force_quirks, ..
        } = token
        {
            // TODO: Implement full stpecs detection
            if force_quirks || name.unwrap_or_default() != "html" {
                return QuirksMode::Quirks;
            }
        }
        QuirksMode::NoQuirks
    }

    fn switch_to(&mut self, mode: InsertMode) {
        if is_trace() {
            println!("Switch to: {:#?}", mode);
        }
        self.insert_mode = mode;
    }

    fn stop_parsing(&mut self) {
        self.should_stop = true;
    }

    fn create_element(&self, tag_token: Token) -> NodePtr {
        let (tag_name, attributes) = if let Token::Tag {
            tag_name,
            attributes,
            ..
        } = tag_token
        {
            (tag_name, attributes)
        } else {
            ("".to_string(), Vec::new())
        };
        let element_ref = dom::create_element(WeakTreeNode::from(&self.document.0), &tag_name);
        for attribute in attributes {
            element_ref
                .as_element()
                .set_attribute(&attribute.name, &attribute.value);
        }
        element_ref
    }

    fn create_element_from_tag_name(&self, tag_name: &str) -> NodePtr {
        self.create_element(Token::Tag {
            tag_name: tag_name.to_owned(),
            self_closing: false,
            attributes: Vec::new(),
            is_end_tag: false,
            self_closing_acknowledged: false,
        })
    }

    fn get_appropriate_place_for_inserting_a_node(
        &self,
        target: Option<NodePtr>,
    ) -> AdjustedInsertionLocation {
        let target = target.unwrap_or(self.open_elements.current_node().unwrap());

        let adjusted_location = if self.foster_parenting
            && match_any!(
                target.as_element().tag_name(),
                "table",
                "tbody",
                "tfoot",
                "thead",
                "tr"
            ) {
            let last_template = self.open_elements.last_element_with_tag_name("template");
            let last_table = self.open_elements.last_element_with_tag_name("table");

            if last_template.is_some()
                && (last_table.is_none() || last_template.unwrap().1 > last_table.unwrap().1)
            {
                // TODO: Fix this to insert to template content when support template tag
                AdjustedInsertionLocation::LastChild(target)
            } else {
                if last_table.is_none() {
                    AdjustedInsertionLocation::LastChild(self.open_elements.get(0))
                } else {
                    if let Some((table, table_index)) = last_table {
                        if let Some(table_parent) = table.parent() {
                            AdjustedInsertionLocation::LastChild(NodePtr(table_parent))
                        } else {
                            let previous_element = self.open_elements.get(table_index - 1);
                            AdjustedInsertionLocation::LastChild(previous_element)
                        }
                    } else {
                        AdjustedInsertionLocation::LastChild(target)
                    }
                }
            }
        } else {
            AdjustedInsertionLocation::LastChild(target)
        };

        // TODO: Fix this to insert to template content when support template tag
        return adjusted_location;
    }

    fn insert_html_element(&mut self, token: Token) -> NodePtr {
        let insert_position = self.get_appropriate_place_for_inserting_a_node(None);
        let element = self.create_element(token);
        let return_ref = element.clone();

        // TODO: check if location is possible to insert node (Idk why so we just leave it for now)
        self.open_elements.push(element.clone());
        self.insert_at(insert_position, element);
        return_ref
    }

    fn insert_at(&mut self, location: AdjustedInsertionLocation, child: NodePtr) {
        match location {
            AdjustedInsertionLocation::LastChild(parent) => parent.append_child(child.0),
            AdjustedInsertionLocation::BeforeSibling(parent, sibling) => {
                parent.insert_before(child.0, Some(sibling.0))
            }
        }
    }

    fn get_node_for_text_insertion(
        &mut self,
        insert_position: AdjustedInsertionLocation,
    ) -> NodePtr {
        match &insert_position {
            AdjustedInsertionLocation::LastChild(parent) => {
                if let Some(last_child) = parent.last_child() {
                    if last_child.as_text_opt().is_some() {
                        return NodePtr(last_child);
                    }
                }
            }
            AdjustedInsertionLocation::BeforeSibling(_, sibling) => {
                if let Some(prev_sibling) = sibling.prev_sibling() {
                    if prev_sibling.as_text_opt().is_some() {
                        return NodePtr(prev_sibling);
                    }
                }
            }
        }
        let text = NodePtr(TreeNode::new(Node::new(NodeData::Text(Text::new(String::new())))));
        text.set_document(WeakTreeNode::from(&self.document.0));
        self.insert_at(insert_position, text.clone());
        return text;
    }

    fn insert_character(&mut self, ch: char) {
        let insert_position = self.get_appropriate_place_for_inserting_a_node(None);
        if insert_position.parent().as_document_opt().is_some() {
            return;
        }

        let text_insertion_node = self.get_node_for_text_insertion(insert_position);

        match &self.text_insertion_node {
            Some(node) if Rc::ptr_eq(node, &text_insertion_node) => {
                self.text_insertion_string_data.push(ch);
            }
            None => {
                self.text_insertion_node = Some(text_insertion_node);
                self.text_insertion_string_data.push(ch);
            }
            _ => {
                self.flush_text_insertion();
                self.text_insertion_node = Some(text_insertion_node);
                self.text_insertion_string_data.push(ch);
            }
        }
    }

    fn flush_text_insertion(&mut self) {
        if self.text_insertion_string_data.is_empty() {
            return;
        }
        if let Some(node) = &self.text_insertion_node {
            let text_node = node.as_text();
            text_node
                .character_data
                .set_data(&self.text_insertion_string_data);

            let parent = node.parent().unwrap();
            let context = ChildrenUpdateContext {
                document: NodePtr(parent.owner_document().unwrap()),
                current_node: node.clone(),
            };
            parent
                .data()
                .as_ref()
                .unwrap()
                .handle_on_children_updated(context);

            self.text_insertion_string_data.clear();
        }
    }

    fn insert_comment(&mut self, data: String) {
        let insert_position = self.get_appropriate_place_for_inserting_a_node(None);
        let comment = TreeNode::new(Node::new(NodeData::Comment(Comment::new(data))));
        comment.set_document(WeakTreeNode::from(&self.document.0));
        self.insert_at(insert_position, NodePtr(comment));
    }

    fn handle_text_only_element(&mut self, token: Token, algorithm: TextOnlyElementParsingAlgo) {
        self.insert_html_element(token);
        match algorithm {
            TextOnlyElementParsingAlgo::GenericRawText => {
                self.tokenizer.switch_to(State::RAWTEXT);
            }
            TextOnlyElementParsingAlgo::GenericRCDataElement => {
                self.tokenizer.switch_to(State::RCDATA);
            }
        }
        self.original_insert_mode = Some(self.insert_mode.clone());
        self.switch_to(InsertMode::Text);
    }

    fn generate_all_implied_end_tags_thoroughly(&mut self) {
        while let Some(node) = self.open_elements.current_node() {
            let tag_name = node.as_element().tag_name();
            if match_any!(
                tag_name, "caption", "colgroup", "dd", "dt", "li", "optgroup", "option", "p", "rb",
                "rt", "rtc", "tbody", "td", "tfoot", "th", "thead", "tr"
            ) {
                self.open_elements.pop();
            } else {
                break;
            }
        }
    }

    fn reset_insertion_mode_appropriately(&mut self) {
        for (index, node) in self.open_elements.0.iter().enumerate().rev() {
            let last = index == 0;

            let node = if self.is_fragment_case {
                self.context_element.clone().unwrap()
            } else {
                node.clone()
            };

            let element = node.as_element();

            if element.tag_name() == "select" {
                for ancestor in self.open_elements.0[0..index].iter().rev() {
                    let ancestor_tag_name = ancestor.as_element().tag_name();
                    if ancestor_tag_name == "template" {
                        self.switch_to(InsertMode::InSelect);
                        return;
                    } else if ancestor_tag_name == "table" {
                        self.switch_to(InsertMode::InSelectInTable);
                        return;
                    }
                }
                self.switch_to(InsertMode::InSelect);
                return;
            }

            if match_any!(element.tag_name(), "td", "th") && !last {
                self.switch_to(InsertMode::InCell);
                return;
            }

            if element.tag_name() == "tr" {
                self.switch_to(InsertMode::InRow);
                return;
            }

            if match_any!(element.tag_name(), "tbody", "thead", "tfoot") && !last {
                self.switch_to(InsertMode::InTableBody);
                return;
            }

            if element.tag_name() == "caption" {
                self.switch_to(InsertMode::InCaption);
                return;
            }

            if element.tag_name() == "colgroup" {
                self.switch_to(InsertMode::InColumnGroup);
                return;
            }

            if element.tag_name() == "table" {
                self.switch_to(InsertMode::InTable);
                return;
            }

            if element.tag_name() == "template" {
                self.switch_to(self.stack_of_template_insert_mode.last().unwrap().clone());
                return;
            }

            if element.tag_name() == "head" {
                self.switch_to(InsertMode::InHead);
                return;
            }

            if element.tag_name() == "body" {
                self.switch_to(InsertMode::InBody);
                return;
            }

            if element.tag_name() == "frameset" {
                self.switch_to(InsertMode::InFrameset);
                return;
            }

            if element.tag_name() == "html" {
                if self.head_pointer.is_none() {
                    self.switch_to(InsertMode::BeforeHead);
                } else {
                    self.switch_to(InsertMode::AfterHead);
                }
                return;
            }

            if last {
                self.switch_to(InsertMode::InBody);
                return;
            }
        }
    }

    fn is_marker_or_open_element(&self, entry: &Entry) -> bool {
        if let Entry::Marker = entry {
            return true;
        }

        if let Entry::Element(element) = entry {
            if self.open_elements.contains_node(&element) {
                return true;
            }
        }

        false
    }

    fn adoption_agency_algo(&mut self, token: &Token) -> AdoptionAgencyOutcome {
        let subject = token.tag_name();

        let current_node = self.current_node();
        if current_node.as_element().tag_name() == *subject {
            if !self.active_formatting_elements.contains_node(&current_node) {
                self.open_elements.pop();
                return AdoptionAgencyOutcome::DoNothing;
            }
        }

        for _ in 0..8 {
            let formatting_element = self
                .active_formatting_elements
                .get_element_after_last_marker(token.tag_name());

            if formatting_element.is_none() {
                return AdoptionAgencyOutcome::RunAnyOtherEndTags;
            }

            let fmt_element = formatting_element.unwrap();

            if !self.open_elements.contains_node(&fmt_element) {
                self.unexpected(&token);
                self.active_formatting_elements.remove_element(&fmt_element);
                return AdoptionAgencyOutcome::DoNothing;
            }

            if self.open_elements.contains_node(&fmt_element)
                && !self.open_elements.has_element_in_scope(&fmt_element)
            {
                self.unexpected(&token);
                return AdoptionAgencyOutcome::DoNothing;
            }

            if !Rc::ptr_eq(&fmt_element, &self.current_node()) {
                self.unexpected(&token);
            }

            let (mut furthest_block, mut furthest_block_index) = {
                let mut found_element = None;
                let mut found_index = None;
                for (index, element) in self.open_elements.iter().rev().enumerate() {
                    if Rc::ptr_eq(&element, &fmt_element) {
                        break;
                    }
                    if is_special_element(&element.as_element().tag_name()) {
                        found_element = Some(element.clone());
                        found_index = Some(index);
                    }
                }
                (found_element, found_index)
            };

            if furthest_block.is_none() {
                while !Rc::ptr_eq(&self.current_node(), &fmt_element) {
                    self.open_elements.pop();
                }
                self.open_elements.pop();
                self.active_formatting_elements.remove_element(&fmt_element);
                return AdoptionAgencyOutcome::DoNothing;
            }

            let mut common_ancestor = {
                let mut found_element = None;
                for (index, el) in self.open_elements.iter().rev().enumerate() {
                    if Rc::ptr_eq(el, &fmt_element) {
                        if index < self.open_elements.len() - 1 {
                            found_element = Some(self.open_elements.get(index - 1));
                        }
                        break;
                    }
                }
                found_element
            };

            let common_ancestor = common_ancestor
                .take()
                .expect("Common ancestor doesn't exists in agency adoption algo");

            let mut bookmark = self
                .active_formatting_elements
                .iter()
                .rposition(|el| match el {
                    Entry::Element(e) => Rc::ptr_eq(e, &fmt_element),
                    _ => false,
                })
                .unwrap();

            let furthest_block = furthest_block.take().unwrap();
            let furthest_block_index = furthest_block_index.take().unwrap();

            let mut node;
            let mut node_index = furthest_block_index;
            let mut last_node = furthest_block.clone();

            let mut inner_counter = 0;

            loop {
                inner_counter += 1;

                node_index -= 1;
                node = self.open_elements.get(node_index);

                if Rc::ptr_eq(&node, &fmt_element) {
                    break;
                }

                if inner_counter > 3 && self.active_formatting_elements.contains_node(&node) {
                    self.active_formatting_elements.remove_element(&node);
                    continue;
                }

                let node_formatting_index = {
                    if let Some(index) = self.active_formatting_elements.get_index_of_node(&node) {
                        index
                    } else {
                        self.open_elements
                            .remove_first_matching(|n| Rc::ptr_eq(n, &node));
                        continue;
                    }
                };

                let node_element = node.as_element();
                let new_element = self.create_element(Token::Tag {
                    tag_name: node_element.tag_name().to_string(),
                    self_closing: false,
                    is_end_tag: false,
                    self_closing_acknowledged: false,
                    attributes: node_element
                        .attributes()
                        .borrow()
                        .iter()
                        .map(|(k, v)| Attribute::from_name_value(k.clone(), v.clone()))
                        .collect(),
                });

                self.open_elements[node_index] = new_element.clone();
                self.active_formatting_elements[node_formatting_index] =
                    Entry::Element(new_element.clone());

                node = new_element;

                if Rc::ptr_eq(&last_node, &furthest_block) {
                    bookmark = node_formatting_index + 1;
                }

                node.append_child(last_node.0);
                last_node = node;
            }

            let insert_place =
                self.get_appropriate_place_for_inserting_a_node(Some(common_ancestor));
            self.insert_at(insert_place, last_node);

            let node_element = node.as_element();
            let new_element = self.create_element(Token::Tag {
                tag_name: node_element.tag_name().to_string(),
                self_closing: false,
                is_end_tag: false,
                self_closing_acknowledged: false,
                attributes: node_element
                    .attributes()
                    .borrow()
                    .iter()
                    .map(|(k, v)| Attribute::from_name_value(k.clone(), v.clone()))
                    .collect(),
            });

            furthest_block.transfer_children_to_node(new_element.0.clone());
            furthest_block.append_child(new_element.0.clone());

            self.active_formatting_elements.remove_element(&fmt_element);
            self.active_formatting_elements[bookmark] = Entry::Element(new_element.clone());
            self.open_elements
                .remove_first_matching(|n| Rc::ptr_eq(n, &fmt_element));
            self.open_elements
                .insert(furthest_block_index + 1, new_element);
        }
        AdoptionAgencyOutcome::DoNothing
    }

    fn unexpected(&self, token: &Token) {
        match token {
            Token::Tag {
                tag_name,
                is_end_tag,
                ..
            } => {
                if *is_end_tag {
                    emit_error!(format!("Unexpected end tag: {}", tag_name))
                } else {
                    emit_error!(format!("Unexpected start tag: {}", tag_name))
                }
            }
            Token::DOCTYPE { .. } => emit_error!("Unexpected DOCTYPE"),
            Token::Comment(_) => emit_error!("Unexpected comment"),
            Token::Character(_) => emit_error!("Unexpected character"),
            Token::EOF => emit_error!("Unexpected EOF"),
        }
    }

    fn close_p_element(&mut self) {
        self.generate_implied_end_tags("p");

        if self
            .open_elements
            .current_node()
            .unwrap()
            .as_element()
            .tag_name()
            != "p"
        {
            emit_error!("Expected p element");
        }

        self.open_elements.pop_until("p");
    }

    fn generate_implied_end_tags(&mut self, exclude: &str) {
        while let Some(node) = self.open_elements.current_node() {
            let element = node.as_element();
            let tag_name = element.tag_name();
            if tag_name != exclude
                && match_any!(
                    tag_name, "dd", "dt", "li", "optgroup", "option", "p", "rb", "rt", "rtc", "rp"
                )
            {
                self.open_elements.pop();
            } else {
                break;
            }
        }
    }

    fn current_node(&self) -> NodePtr {
        self.open_elements.current_node().unwrap()
    }

    fn reconstruct_active_formatting_elements(&mut self) {
        if self.active_formatting_elements.len() == 0 {
            return;
        }

        let last_active_element = self.active_formatting_elements.last().unwrap();

        if self.is_marker_or_open_element(last_active_element) {
            return;
        }

        let mut last_index = self.active_formatting_elements.len() - 1;

        loop {
            if last_index == 0 {
                break;
            }

            last_index -= 1;
            let entry = &self.active_formatting_elements[last_index];

            if self.is_marker_or_open_element(entry) {
                last_index += 1;
                break;
            }
        }

        loop {
            let element = match &self.active_formatting_elements[last_index] {
                Entry::Element(element) => element.clone(),
                Entry::Marker => panic!("Unexpected marker while building DOM tree!"),
            };

            let new_element = {
                let element = element.as_element();

                self.insert_html_element(Token::Tag {
                    is_end_tag: false,
                    self_closing: false,
                    self_closing_acknowledged: false,
                    tag_name: element.tag_name().to_string(),
                    attributes: element
                        .attributes()
                        .borrow()
                        .iter()
                        .map(|entry| Attribute {
                            name: entry.0.clone(),
                            value: entry.1.clone(),
                            prefix: String::new(),
                            namespace: String::new(),
                        })
                        .collect(),
                })
            };

            self.active_formatting_elements[last_index] = Entry::Element(new_element);

            if last_index == self.active_formatting_elements.len() - 1 {
                break;
            }
            last_index += 1;
        }
    }

    fn close_cell(&mut self) {
        self.generate_implied_end_tags("");
        let current_tag_name = self.current_node().as_element().tag_name();
        if current_tag_name != "td" || current_tag_name != "th" {
            emit_error!("Unexpected node encountered while closing cell");
        }
        self.open_elements.pop_until_match(|element| {
            let tag_name = element.tag_name();
            return tag_name == "td" || tag_name == "th";
        });
        self.active_formatting_elements.clear_up_to_last_marker();
        self.switch_to(InsertMode::InRow);
    }
}

impl<T: Tokenizing> TreeBuilder<T> {
    fn handle_initial(&mut self, token: Token) {
        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return;
            }
        }

        if let Token::Comment(data) = token {
            let comment = TreeNode::new(Node::new(NodeData::Comment(Comment::new(data))));
            comment.set_document(WeakTreeNode::from(&self.document.0));
            self.document.append_child(comment);
            return;
        }

        if let Token::DOCTYPE {
            name,
            public_identifier,
            system_identifier,
            ..
        } = token.clone()
        {
            let name = name.unwrap_or_default();

            let error = match (
                name.as_str(),
                public_identifier.clone(),
                system_identifier.clone(),
            ) {
                ("html", None, None) => false,
                ("html", None, Some(c)) if c == "about:legacy-compat" => false,
                _ => true,
            };

            if error {
                self.unexpected(&token)
            }

            let doctype = DocumentType::new(name, public_identifier, system_identifier);

            if let Some(doc) = self.document.as_document_opt() {
                doc.set_doctype(doctype);
                doc.set_mode(self.which_quirks_mode(token));
            }

            self.switch_to(InsertMode::BeforeHtml);
            return;
        }

        self.unexpected(&token);
        self.switch_to(InsertMode::BeforeHtml);
        self.process(token)
    }

    fn handle_before_html(&mut self, token: Token) {
        fn anything_else<T: Tokenizing>(this: &mut TreeBuilder<T>, token: Token) {
            let element = this.create_element_from_tag_name("html");
            this.document.append_child(element.0.clone());
            this.open_elements.push(element.clone());
            // TODO: Implement additional steps in specs
            this.switch_to(InsertMode::BeforeHead);
            this.process(token.clone())
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if let Token::Comment(data) = token {
            let comment = TreeNode::new(Node::new(NodeData::Comment(Comment::new(data))));
            self.document.append_child(comment);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            let element = self.create_element(token);
            self.document.append_child(element.0.clone());
            self.open_elements.push(element.clone());
            // TODO: implement additional steps in specs
            self.switch_to(InsertMode::BeforeHead);
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "head", "body", "html", "br") {
            anything_else(self, token);
            return;
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            anything_else(self, token);
            return;
        }

        anything_else(self, token);
    }

    fn handle_before_head(&mut self, token: Token) {
        fn anything_else<T: Tokenizing>(this: &mut TreeBuilder<T>, token: Token) {
            let head_element = this.insert_html_element(Token::Tag {
                tag_name: "head".to_owned(),
                attributes: Vec::new(),
                is_end_tag: false,
                self_closing: false,
                self_closing_acknowledged: false,
            });
            this.head_pointer = Some(head_element.clone());
            this.switch_to(InsertMode::InHead);
            this.process(token)
        }

        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return;
            }
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_start_tag() && token.tag_name() == "head" {
            let head_element = self.insert_html_element(token);
            self.head_pointer = Some(head_element);
            self.switch_to(InsertMode::InHead);
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "head", "body", "html", "br") {
            return anything_else(self, token);
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }

        anything_else(self, token);
    }

    fn handle_in_head(&mut self, mut token: Token) {
        if let Token::Character(c) = token {
            if is_whitespace(c) {
                self.insert_character(c);
                return;
            }
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_start_tag()
            && match_any!(token.tag_name(), "base", "basefont", "bgsound", "link")
        {
            self.insert_html_element(token.clone());
            self.open_elements.pop();
            token.acknowledge_self_closing_if_set();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "meta" {
            self.insert_html_element(token.clone());
            self.open_elements.pop();
            token.acknowledge_self_closing_if_set();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "title" {
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRCDataElement);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "noscript" && self.scripting {
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "noframes", "style") {
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "noscript" && !self.scripting {
            self.insert_html_element(token);
            self.switch_to(InsertMode::InHeadNoScript);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "script" {
            let insert_position = self.get_appropriate_place_for_inserting_a_node(None);
            let element = self.create_element(token);

            // TODO: implement step 5

            self.insert_at(insert_position, element.clone());
            self.open_elements.push(element.clone());
            self.tokenizer.switch_to(State::ScriptData);
            self.original_insert_mode = Some(self.insert_mode.clone());
            self.switch_to(InsertMode::Text);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "head" {
            self.open_elements.pop();
            self.switch_to(InsertMode::AfterHead);
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "body", "html", "br") {
            self.open_elements.pop();
            self.switch_to(InsertMode::AfterHead);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "template" {
            self.insert_html_element(token);
            self.active_formatting_elements.add_marker();
            self.frameset_ok = false;
            self.switch_to(InsertMode::InTemplate);
            self.stack_of_template_insert_mode
                .push(InsertMode::InTemplate);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "template" {
            if !self.open_elements.contains("template") {
                emit_error!("No template tag found");
                return;
            }

            self.generate_all_implied_end_tags_thoroughly();

            if let Some(node) = self.open_elements.current_node() {
                let element = node.as_element();
                if element.tag_name() != "template" {
                    emit_error!("Expected current node to be template");
                }
            }

            self.open_elements.pop_until("template");
            self.active_formatting_elements.clear_up_to_last_marker();
            self.stack_of_template_insert_mode.pop();
            self.reset_insertion_mode_appropriately();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "head" {
            self.unexpected(&token);
            return;
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }

        self.open_elements.pop();
        self.switch_to(InsertMode::AfterHead);
        self.process(token)
    }

    fn handle_in_head_no_script(&mut self, token: Token) {
        fn anything_else<T: Tokenizing>(this: &mut TreeBuilder<T>, token: Token) {
            this.unexpected(&token);
            this.open_elements.pop();
            this.switch_to(InsertMode::InHead);
            this.process(token)
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_end_tag() && token.tag_name() == "noscript" {
            self.open_elements.pop();
            self.switch_to(InsertMode::InHead);
            return;
        }

        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return self.handle_in_head(token);
            }
        }

        if let Token::Comment(_) = token {
            return self.handle_in_head(token);
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "basefont",
                "bgsound",
                "link",
                "meta",
                "noframes",
                "style"
            )
        {
            return self.handle_in_head(token);
        }

        if token.is_end_tag() && token.tag_name() == "br" {
            return anything_else(self, token);
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "head", "noscript") {
            self.unexpected(&token);
            return;
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }

        anything_else(self, token)
    }

    fn handle_after_head(&mut self, token: Token) {
        fn anything_else<T: Tokenizing>(this: &mut TreeBuilder<T>, token: Token) {
            this.insert_html_element(Token::Tag {
                is_end_tag: false,
                tag_name: "body".to_owned(),
                self_closing: false,
                self_closing_acknowledged: false,
                attributes: Vec::new(),
            });
            this.switch_to(InsertMode::InBody);
            this.process(token)
        }

        if let Token::Character(c) = token {
            if is_whitespace(c) {
                self.insert_character(c);
                return;
            }
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_start_tag() && token.tag_name() == "body" {
            self.insert_html_element(token);
            self.frameset_ok = false;
            self.switch_to(InsertMode::InBody);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "frameset" {
            self.insert_html_element(token);
            self.switch_to(InsertMode::InFrameset);
            return;
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "base",
                "basefont",
                "bgsound",
                "link",
                "meta",
                "noframes",
                "script",
                "style",
                "template",
                "title"
            )
        {
            self.unexpected(&token);
            let head = self.head_pointer.clone().unwrap();
            self.open_elements.push(head.clone());
            self.handle_in_head(token);
            self.open_elements
                .remove_first_matching(|node| Rc::ptr_eq(node, &head));
            return;
        }

        if token.is_end_tag() && token.tag_name() == "template" {
            return self.handle_in_head(token);
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "body", "html", "br") {
            return anything_else(self, token);
        }

        if token.is_start_tag() && token.tag_name() == "head" {
            self.unexpected(&token);
            return;
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }

        anything_else(self, token)
    }

    fn handle_in_body(&mut self, mut token: Token) {
        fn any_other_end_tags<T: Tokenizing>(this: &mut TreeBuilder<T>, token: Token) {
            let mut index: Option<usize> = None;
            for (idx, node) in this.open_elements.0.iter().enumerate().rev() {
                let current_tag_name = node.as_element().tag_name();
                if current_tag_name == *token.tag_name() {
                    if Rc::ptr_eq(&node, &this.current_node()) {
                        this.unexpected(&token);
                    }
                    index = Some(idx);
                    break;
                }

                if is_special_element(&current_tag_name) {
                    emit_error!("Unexpected special element");
                    return;
                }
            }

            let match_idx = match index {
                Some(idx) => idx,
                None => {
                    this.unexpected(&token);
                    return;
                }
            };

            this.generate_implied_end_tags(token.tag_name());

            while this.open_elements.len() > match_idx {
                this.open_elements.pop();
            }
        }

        if let Token::Character(c) = token {
            if c == '\0' {
                emit_error!("Unexpected null character");
                return;
            }

            if is_whitespace(c) {
                self.reconstruct_active_formatting_elements();
                self.insert_character(c);
                return;
            }

            self.reconstruct_active_formatting_elements();
            self.insert_character(c);
            self.frameset_ok = false;
            return;
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            emit_error!("Unexpected DOCTYPE");
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            emit_error!("Unexpected HTML tag");
            if self.open_elements.contains("template") {
                return;
            }

            let current_node = self.open_elements.current_node().unwrap();
            let current_element = current_node.as_element();

            for attribute in token.attributes() {
                if current_element.has_attribute(&attribute.name) {
                    continue;
                }
                current_element.set_attribute(&attribute.name, &attribute.value);
            }
            return;
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "base",
                "basefont",
                "bgsound",
                "link",
                "meta",
                "noframes",
                "script",
                "style",
                "template",
                "title"
            )
        {
            return self.handle_in_head(token);
        }

        if token.is_end_tag() && token.tag_name() == "template" {
            return self.handle_in_head(token);
        }

        if token.is_start_tag() && token.tag_name() == "body" {
            self.unexpected(&token);
            if self.open_elements.len() == 1 {
                return;
            }

            if let Some(element) = self.open_elements.get(1).as_element_opt() {
                if element.tag_name() != "body" {
                    return;
                }
            }

            if self.open_elements.contains("template") {
                return;
            }

            self.frameset_ok = false;
            let body = self.open_elements.get(1);
            let body = body.as_element();
            for attribute in token.attributes() {
                if body.has_attribute(&attribute.name) {
                    continue;
                }
                body.set_attribute(&attribute.name, &attribute.value);
            }
        }

        if token.is_start_tag() && token.tag_name() == "frameset" {
            self.unexpected(&token);
            if self.open_elements.len() == 1 {
                return;
            }

            if let Some(element) = self.open_elements.get(1).as_element_opt() {
                if element.tag_name() != "body" {
                    return;
                }
            }

            if !self.frameset_ok {
                return;
            }

            let second_element = self.open_elements.get(1);
            second_element.detach();

            while self.current_node().as_element().tag_name() != "html" {
                self.open_elements.pop();
            }

            self.insert_html_element(token);
            self.switch_to(InsertMode::InFrameset);
            return;
        }

        if token.is_eof() {
            if self.stack_of_template_insert_mode.len() > 0 {
                return self.handle_in_template(token);
            }

            if self.open_elements.any(|node| {
                !(match_any!(
                    node.as_element().tag_name(),
                    "dd",
                    "dt",
                    "li",
                    "optgroup",
                    "option",
                    "p",
                    "rb",
                    "rp",
                    "rt",
                    "rtc",
                    "tbody",
                    "td",
                    "tfoot",
                    "th",
                    "thead",
                    "tr",
                    "body",
                    "html"
                ))
            }) {
                self.unexpected(&token);
            }

            self.stop_parsing();
            return;
        }

        if token.is_end_tag() && token.tag_name() == "body" {
            if self.open_elements.has_element_name_in_scope("body") {
                self.unexpected(&token);
                return;
            }
            if self.open_elements.any(|node| {
                !(match_any!(
                    node.as_element().tag_name(),
                    "dd",
                    "dt",
                    "li",
                    "optgroup",
                    "option",
                    "p",
                    "rb",
                    "rp",
                    "rt",
                    "rtc",
                    "tbody",
                    "td",
                    "tfoot",
                    "th",
                    "thead",
                    "tr",
                    "body",
                    "html"
                ))
            }) {
                self.unexpected(&token);
            }
            self.switch_to(InsertMode::AfterBody);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "html" {
            if self.open_elements.has_element_name_in_scope("body") {
                self.unexpected(&token);
                return;
            }
            if self.open_elements.any(|node| {
                !(match_any!(
                    node.as_element().tag_name(),
                    "dd",
                    "dt",
                    "li",
                    "optgroup",
                    "option",
                    "p",
                    "rb",
                    "rp",
                    "rt",
                    "rtc",
                    "tbody",
                    "td",
                    "tfoot",
                    "th",
                    "thead",
                    "tr",
                    "body",
                    "html"
                ))
            }) {
                self.unexpected(&token);
            }
            self.switch_to(InsertMode::AfterBody);
            return self.process(token);
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "address",
                "article",
                "aside",
                "blockquote",
                "center",
                "details",
                "dialog",
                "dir",
                "div",
                "dl",
                "fieldset",
                "figcaption",
                "figure",
                "footer",
                "header",
                "hgroup",
                "main",
                "menu",
                "nav",
                "ol",
                "p",
                "section",
                "summary",
                "ul"
            )
        {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }

            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "h1", "h2", "h3", "h4", "h5", "h6")
        {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }

            let current_tag_name = self
                .open_elements
                .current_node()
                .unwrap()
                .as_element()
                .tag_name();

            if match_any!(current_tag_name, "h1", "h2", "h3", "h4", "h5", "h6") {
                self.unexpected(&token);
                self.open_elements.pop();
            }

            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "pre", "listing") {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }

            self.insert_html_element(token);

            let next_token = self.tokenizer.next_token();

            self.frameset_ok = false;

            if let Token::Character(c) = next_token {
                if c == '\n' {
                    // ignore the token
                } else {
                    self.process(next_token);
                }
            }
            return;
        }

        if token.is_start_tag() && token.tag_name() == "form" {
            let has_template_on_stack = self.open_elements.contains("template");
            if self.form_pointer.is_some() && !has_template_on_stack {
                self.unexpected(&token);
                return;
            }

            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }

            let form_element = self.insert_html_element(token);
            if !has_template_on_stack {
                self.form_pointer = Some(form_element);
            }
            return;
        }

        if token.is_start_tag() && token.tag_name() == "li" {
            self.frameset_ok = false;
            for node in self.open_elements.0.iter().rev() {
                let element_tag_name = node.as_element().tag_name();
                if element_tag_name == "li" {
                    self.generate_implied_end_tags("li");
                    if self.current_node().as_element().tag_name() != "li" {
                        emit_error!("Expected 'li' tag");
                    }
                    self.open_elements.pop_until("li");
                    break;
                }

                if !match_any!(element_tag_name, "address", "div", "p")
                    && is_special_element(&element_tag_name)
                {
                    break;
                }
            }

            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }
            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "dd", "dt") {
            self.frameset_ok = false;
            for node in self.open_elements.0.iter().rev() {
                let element_tag_name = node.as_element().tag_name();
                if element_tag_name == "dd" {
                    self.generate_implied_end_tags("dd");
                    if self.current_node().as_element().tag_name() != "dd" {
                        emit_error!("Expected 'dd' tag");
                    }
                    self.open_elements.pop_until("dd");
                    break;
                }

                if element_tag_name == "dt" {
                    self.generate_implied_end_tags("dt");
                    if self.current_node().as_element().tag_name() != "dt" {
                        emit_error!("Expected 'dt' tag");
                    }
                    self.open_elements.pop_until("dt");
                    break;
                }

                if !match_any!(element_tag_name, "address", "div", "p")
                    && is_special_element(&element_tag_name)
                {
                    break;
                }
            }

            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }
            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "plaintext" {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }
            self.insert_html_element(token);
            self.tokenizer.switch_to(State::PLAINTEXT);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "button" {
            if self.open_elements.has_element_name_in_scope("button") {
                self.unexpected(&token);
                self.generate_implied_end_tags("");
                self.open_elements.pop_until("button");
            }
            self.reconstruct_active_formatting_elements();
            self.insert_html_element(token);
            self.frameset_ok = false;
            return;
        }

        if token.is_end_tag()
            && match_any!(
                token.tag_name(),
                "address",
                "article",
                "aside",
                "blockquote",
                "button",
                "center",
                "details",
                "dialog",
                "dir",
                "div",
                "dl",
                "fieldset",
                "figcaption",
                "figure",
                "footer",
                "header",
                "hgroup",
                "listing",
                "main",
                "menu",
                "nav",
                "ol",
                "pre",
                "section",
                "summary",
                "ul"
            )
        {
            if !self
                .open_elements
                .has_element_name_in_scope(&token.tag_name())
            {
                self.unexpected(&token);
                return;
            }

            self.generate_implied_end_tags("");
            if self.current_node().as_element().tag_name() != *token.tag_name() {
                self.unexpected(&token);
                return;
            }
            self.open_elements.pop_until(&token.tag_name());
            return;
        }

        if token.is_end_tag() && token.tag_name() == "form" {
            if !self.open_elements.contains("template") {
                let node = self.form_pointer.clone();
                self.form_pointer = None;

                if node.is_none() {
                    self.unexpected(&token);
                    return;
                }

                let node = node.unwrap();

                if self.open_elements.has_element_in_scope(&node) {
                    self.unexpected(&token);
                    return;
                }

                self.generate_implied_end_tags("");

                if !Rc::ptr_eq(&self.current_node(), &node) {
                    self.unexpected(&token);
                }

                self.open_elements
                    .remove_first_matching(|fnode| Rc::ptr_eq(fnode, &node));
            } else {
                if !self.open_elements.has_element_name_in_scope("form") {
                    self.unexpected(&token);
                    return;
                }
                self.generate_implied_end_tags("");
                if self.current_node().as_element().tag_name() != "form" {
                    self.unexpected(&token);
                }
                self.open_elements.pop_until("form");
            }
            return;
        }

        if token.is_end_tag() && token.tag_name() == "p" {
            if !self.open_elements.has_element_name_in_button_scope("p") {
                self.unexpected(&token);
                self.insert_html_element(Token::new_start_tag_with_name("p"));
            }
            self.close_p_element();
            return;
        }

        if token.is_end_tag() && token.tag_name() == "li" {
            if !self.open_elements.has_element_name_in_list_item_scope("li") {
                self.unexpected(&token);
                return;
            }

            self.generate_implied_end_tags("li");
            if self.current_node().as_element().tag_name() != "li" {
                self.unexpected(&token);
            }
            self.open_elements.pop_until("li");
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "dd", "dt") {
            if !self
                .open_elements
                .has_element_name_in_scope(&token.tag_name())
            {
                self.unexpected(&token);
                return;
            }
            self.generate_implied_end_tags(&token.tag_name());
            if self.current_node().as_element().tag_name() != *token.tag_name() {
                self.unexpected(&token);
            }
            self.open_elements.pop_until(&token.tag_name());
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "h1", "h2", "h3", "h4", "h5", "h6") {
            if !self.open_elements.has_element_name_in_scope("h1")
                && !self.open_elements.has_element_name_in_scope("h2")
                && !self.open_elements.has_element_name_in_scope("h3")
                && !self.open_elements.has_element_name_in_scope("h4")
                && !self.open_elements.has_element_name_in_scope("h5")
                && !self.open_elements.has_element_name_in_scope("h6")
            {
                self.unexpected(&token);
                return;
            }
            self.generate_implied_end_tags("");
            if self.current_node().as_element().tag_name() != *token.tag_name() {
                self.unexpected(&token);
            }
            self.open_elements.pop_until_match(|element| {
                match_any!(element.tag_name(), "h1", "h2", "h3", "h4", "h5", "h6")
            });
            return;
        }

        if token.is_start_tag() && token.tag_name() == "a" {
            if let Some(el) = self
                .active_formatting_elements
                .get_element_after_last_marker("a")
            {
                self.unexpected(&token);
                match self.adoption_agency_algo(&token) {
                    AdoptionAgencyOutcome::DoNothing => {}
                    AdoptionAgencyOutcome::RunAnyOtherEndTags => {
                        return any_other_end_tags(self, token);
                    }
                }
                self.active_formatting_elements.remove_element(&el);
                self.open_elements
                    .remove_first_matching(|fnode| Rc::ptr_eq(fnode, &el));
            }
            self.reconstruct_active_formatting_elements();
            let element = self.insert_html_element(token);
            self.active_formatting_elements
                .push(Entry::Element(element));
            return;
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "b",
                "big",
                "code",
                "em",
                "font",
                "i",
                "s",
                "small",
                "strike",
                "strong",
                "tt",
                "u"
            )
        {
            self.reconstruct_active_formatting_elements();
            let element = self.insert_html_element(token);
            self.active_formatting_elements
                .push(Entry::Element(element));
            return;
        }

        if token.is_start_tag() && token.tag_name() == "nobr" {
            self.reconstruct_active_formatting_elements();
            if self.open_elements.has_element_name_in_scope("nobr") {
                self.unexpected(&token);
                match self.adoption_agency_algo(&token) {
                    AdoptionAgencyOutcome::DoNothing => {}
                    AdoptionAgencyOutcome::RunAnyOtherEndTags => {
                        return any_other_end_tags(self, token);
                    }
                }
                self.reconstruct_active_formatting_elements();
            }
            let element = self.insert_html_element(token);
            self.active_formatting_elements
                .push(Entry::Element(element));
            return;
        }

        if token.is_end_tag()
            && match_any!(
                token.tag_name(),
                "a",
                "b",
                "big",
                "code",
                "em",
                "font",
                "i",
                "nobr",
                "s",
                "small",
                "strike",
                "strong",
                "tt",
                "u"
            )
        {
            match self.adoption_agency_algo(&token) {
                AdoptionAgencyOutcome::RunAnyOtherEndTags => any_other_end_tags(self, token),
                _ => {}
            }

            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "applet", "marquee", "object") {
            self.reconstruct_active_formatting_elements();
            self.insert_html_element(token);
            self.active_formatting_elements.add_marker();
            self.frameset_ok = false;
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "applet", "marquee", "object") {
            if !self
                .open_elements
                .has_element_name_in_scope(&token.tag_name())
            {
                self.unexpected(&token);
                return;
            }
            self.generate_implied_end_tags("");
            if self.current_node().as_element().tag_name() != *token.tag_name() {
                self.unexpected(&token);
            }
            self.open_elements.pop_until(token.tag_name());
            self.active_formatting_elements.clear_up_to_last_marker();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "table" {
            let document = self.document.clone();
            let document = document.as_document();
            if let QuirksMode::NoQuirks = document.get_mode() {
                if self.open_elements.has_element_name_in_button_scope("p") {
                    self.close_p_element();
                }
            }
            self.insert_html_element(token);
            self.frameset_ok = false;
            self.switch_to(InsertMode::InTable);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "br" {
            self.unexpected(&token);
            token.drop_attributes();
            self.reconstruct_active_formatting_elements();
            token.acknowledge_self_closing_if_set();
            self.insert_html_element(token);
            self.open_elements.pop();
            self.frameset_ok = false;
            return;
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "area",
                "br",
                "embed",
                "img",
                "keygen",
                "wbr"
            )
        {
            self.reconstruct_active_formatting_elements();
            token.acknowledge_self_closing_if_set();
            self.insert_html_element(token);
            self.open_elements.pop();
            self.frameset_ok = false;
            return;
        }

        if token.is_start_tag() && token.tag_name() == "input" {
            self.reconstruct_active_formatting_elements();
            token.acknowledge_self_closing_if_set();
            self.insert_html_element(token.clone());
            self.open_elements.pop();
            if token.attribute("type").is_none() {
                self.frameset_ok = false;
                return;
            }

            if let Some(value) = token.attribute("type") {
                if !value.eq_ignore_ascii_case("hidden") {
                    self.frameset_ok = false;
                }
            }
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "param", "source", "track") {
            token.acknowledge_self_closing_if_set();
            self.insert_html_element(token);
            self.open_elements.pop();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "hr" {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }
            token.acknowledge_self_closing_if_set();
            self.insert_html_element(token);
            self.open_elements.pop();
            self.frameset_ok = false;
            return;
        }

        if token.is_start_tag() && token.tag_name() == "image" {
            self.unexpected(&token);
            token.set_tag_name("img"); // But why?? :troll:
            return self.process(token);
        }

        if token.is_start_tag() && token.tag_name() == "textarea" {
            self.insert_html_element(token);
            let next_token = self.tokenizer.next_token();
            self.tokenizer.switch_to(State::RCDATA);
            self.original_insert_mode = Some(self.insert_mode.clone());
            self.frameset_ok = false;
            self.switch_to(InsertMode::Text);
            if let Token::Character(c) = next_token {
                if c == '\n' {
                    return;
                }
            }
            return self.process(next_token);
        }

        if token.is_start_tag() && token.tag_name() == "xmp" {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }
            self.reconstruct_active_formatting_elements();
            self.frameset_ok = false;
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "iframe" {
            self.frameset_ok = false;
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "noembed" {
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "noscript" && self.scripting {
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "select" {
            self.reconstruct_active_formatting_elements();
            self.insert_html_element(token);
            self.frameset_ok = false;
            match self.insert_mode {
                InsertMode::InTable
                | InsertMode::InCaption
                | InsertMode::InTableBody
                | InsertMode::InRow
                | InsertMode::InCell => self.switch_to(InsertMode::InSelectInTable),
                _ => self.switch_to(InsertMode::InSelect),
            }
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "optgroup", "option") {
            if self.current_node().as_element().tag_name() == "option" {
                self.open_elements.pop();
            }
            self.reconstruct_active_formatting_elements();
            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "rb", "rtc") {
            if self.open_elements.has_element_name_in_scope("ruby") {
                self.generate_implied_end_tags("");
                if self.current_node().as_element().tag_name() != "ruby" {
                    self.unexpected(&token);
                }
            }
            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "rp", "rt") {
            if self.open_elements.has_element_name_in_scope("ruby") {
                self.generate_implied_end_tags("rtc");
                let current_tag_name = self.current_node().as_element().tag_name();
                if current_tag_name != "ruby" || current_tag_name != "rtc" {
                    self.unexpected(&token);
                }
            }
            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "math" {
            // TODO: support math
            unimplemented!();
        }

        if token.is_start_tag() && token.tag_name() == "svg" {
            self.reconstruct_active_formatting_elements();
            adjust_svg_attributes(&mut token);
            adjust_foreign_attributes(&mut token);

            if token.is_self_closing() {
                self.open_elements.pop();
                token.acknowledge_self_closing_if_set();
            }

            // TODO: change this to insert foreign element
            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "caption",
                "col",
                "colgroup",
                "frame",
                "head",
                "tbody",
                "td",
                "tfoot",
                "th",
                "thead",
                "tr"
            )
        {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() {
            self.reconstruct_active_formatting_elements();
            self.insert_html_element(token);
            return;
        }

        if token.is_end_tag() {
            return any_other_end_tags(self, token);
        }
    }

    fn handle_text(&mut self, token: Token) {
        if let Token::Character(c) = token {
            self.insert_character(c);
            return;
        }

        if let Token::EOF = token {
            self.unexpected(&token);
            self.open_elements.pop();
            self.switch_to(self.original_insert_mode.clone().unwrap());
            return self.process(token);
        }

        if token.is_end_tag() && token.tag_name() == "script" {
            // TODO: support script tag
            self.flush_text_insertion();
        }

        if token.is_end_tag() {
            self.open_elements.pop();
            self.switch_to(self.original_insert_mode.clone().unwrap());
            return;
        }
    }

    fn handle_in_table(&mut self, mut token: Token) {
        if let Token::Character(_) = token {
            if match_any!(
                self.current_node().as_element().tag_name(),
                "table",
                "tbody",
                "tfoot",
                "thead",
                "tr"
            ) {
                self.table_character_tokens.clear();
                self.original_insert_mode = Some(self.insert_mode.clone());
                self.switch_to(InsertMode::InTableText);
                return self.process(token);
            }
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "caption" {
            self.open_elements.clear_back_to_table_context();
            self.active_formatting_elements.add_marker();
            self.insert_html_element(token);
            self.switch_to(InsertMode::InCaption);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "colgroup" {
            self.open_elements.clear_back_to_table_context();
            self.insert_html_element(token);
            self.switch_to(InsertMode::InColumnGroup);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "col" {
            self.open_elements.clear_back_to_table_context();
            self.insert_html_element(Token::new_start_tag_with_name("colgroup"));
            self.switch_to(InsertMode::InColumnGroup);
            return self.process(token);
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "tbody", "tfoot", "thead") {
            self.open_elements.clear_back_to_table_context();
            self.insert_html_element(token);
            self.switch_to(InsertMode::InTableBody);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "td", "th", "tr") {
            self.open_elements.clear_back_to_table_context();
            self.insert_html_element(Token::new_start_tag_with_name("tbody"));
            self.switch_to(InsertMode::InTableBody);
            return self.process(token);
        }

        if token.is_start_tag() && token.tag_name() == "table" {
            self.unexpected(&token);
            if !self.open_elements.has_element_name_in_table_scope("table") {
                return;
            }
            self.open_elements.pop_until("table");
            self.reset_insertion_mode_appropriately();
            return self.process(token);
        }

        if token.is_end_tag() && token.tag_name() == "table" {
            if !self.open_elements.has_element_name_in_table_scope("table") {
                self.unexpected(&token);
                return;
            }
            self.open_elements.pop_until("table");
            self.reset_insertion_mode_appropriately();
            return;
        }

        if token.is_end_tag()
            && match_any!(
                token.tag_name(),
                "body",
                "caption",
                "col",
                "colgroup",
                "html",
                "tbody",
                "td",
                "tfoot",
                "th",
                "thead",
                "tr"
            )
        {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "style", "script", "template") {
            self.handle_in_head(token);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "template" {
            self.handle_in_head(token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "input" {
            if let Some(value) = token.attribute("type") {
                if !value.eq_ignore_ascii_case("hidden") {
                    self.unexpected(&token);
                    self.foster_parenting = true;
                    self.handle_in_body(token);
                    self.foster_parenting = false;
                    return;
                }
            } else {
                self.unexpected(&token);
                self.foster_parenting = true;
                self.handle_in_body(token);
                self.foster_parenting = false;
                return;
            }

            self.unexpected(&token);
            token.acknowledge_self_closing_if_set();
            let element = self.insert_html_element(token);
            self.open_elements
                .remove_first_matching(|el| Rc::ptr_eq(el, &element));
            return;
        }

        if token.is_start_tag() && token.tag_name() == "form" {
            self.unexpected(&token);
            if self.open_elements.contains("template") || self.form_pointer.is_some() {
                return;
            }

            let element = self.insert_html_element(token);
            self.form_pointer = Some(element.clone());
            self.open_elements
                .remove_first_matching(|el| Rc::ptr_eq(el, &element));
            return;
        }

        if let Token::EOF = token {
            return self.handle_in_body(token);
        }

        self.unexpected(&token);
        self.foster_parenting = true;
        self.handle_in_body(token);
        self.foster_parenting = false;
    }

    fn handle_in_table_text(&mut self, token: Token) {
        if let Token::Character(c) = token {
            if c == '\0' {
                self.unexpected(&token);
                return;
            }
            self.table_character_tokens.push(token);
            return;
        }
        let has_non_whitespace_char =
            self.table_character_tokens
                .iter()
                .any(|c_token| match c_token {
                    Token::Character(c) if !is_whitespace(*c) => true,
                    _ => false,
                });

        if has_non_whitespace_char {
            emit_error!("Non-whitespace in table text");
            let table_character_tokens = self.table_character_tokens.clone();
            for c_token in table_character_tokens {
                self.foster_parenting = true;
                self.handle_in_body(c_token.clone());
                self.foster_parenting = false;
            }
        } else {
            let table_character_tokens = self.table_character_tokens.clone();
            for c_token in table_character_tokens {
                if let Token::Character(c) = c_token {
                    self.insert_character(c);
                }
            }
        }

        self.switch_to(self.original_insert_mode.clone().unwrap());
    }

    fn handle_in_caption(&mut self, token: Token) {
        if token.is_end_tag() && token.tag_name() == "caption" {
            if self
                .open_elements
                .has_element_name_in_table_scope("caption")
            {
                self.unexpected(&token);
                return;
            }
            self.generate_implied_end_tags("");
            if self.current_node().as_element().tag_name() != "caption" {
                self.unexpected(&token);
            }
            self.open_elements.pop_until("caption");
            self.active_formatting_elements.clear_up_to_last_marker();
            self.switch_to(InsertMode::InTable);
            return;
        }

        if (token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "caption",
                "col",
                "colgroup",
                "tbody",
                "td",
                "tfoot",
                "th",
                "thead",
                "tr"
            ))
            || (token.is_end_tag() && token.tag_name() == "table")
        {
            if self
                .open_elements
                .has_element_name_in_table_scope("caption")
            {
                self.unexpected(&token);
                return;
            }
            self.generate_implied_end_tags("");
            if self.current_node().as_element().tag_name() != "caption" {
                self.unexpected(&token);
            }
            self.open_elements.pop_until("caption");
            self.active_formatting_elements.clear_up_to_last_marker();
            self.switch_to(InsertMode::InTable);
            return self.process(token);
        }

        if token.is_end_tag()
            && match_any!(
                token.tag_name(),
                "body",
                "col",
                "colgroup",
                "html",
                "tbody",
                "td",
                "tfoot",
                "th",
                "thead",
                "tr"
            )
        {
            self.unexpected(&token);
            return;
        }

        return self.handle_in_body(token);
    }

    fn handle_after_body(&mut self, token: Token) {
        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return self.handle_in_body(token);
            }
        }

        if let Token::Comment(data) = token {
            let comment = TreeNode::new(Node::new(NodeData::Comment(Comment::new(data))));
            let html_el = self.open_elements.get(0);
            html_el.append_child(comment);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_end_tag() && token.tag_name() == "html" {
            if self.is_fragment_case {
                self.unexpected(&token);
                return;
            }
            self.switch_to(InsertMode::AfterAfterBody);
        }

        if let Token::EOF = token {
            self.stop_parsing();
            return;
        }

        self.unexpected(&token);
        self.switch_to(InsertMode::InBody);
        return self.process(token);
    }

    fn handle_after_after_body(&mut self, token: Token) {
        if let Token::Comment(data) = token {
            let comment = TreeNode::new(Node::new(NodeData::Comment(Comment::new(data))));
            self.document.append_child(comment);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            return self.handle_in_body(token);
        }

        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return self.handle_in_body(token);
            }
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if let Token::EOF = token {
            self.stop_parsing();
            return;
        }

        self.unexpected(&token);
        self.switch_to(InsertMode::InBody);
        return self.process(token);
    }

    fn handle_in_column_group(&mut self, mut token: Token) {
        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return self.insert_character(c);
            }
        }

        if let Token::Comment(data) = token {
            return self.insert_comment(data);
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_start_tag() && token.tag_name() == "col" {
            token.acknowledge_self_closing_if_set();
            self.insert_html_element(token);
            self.open_elements.pop();
            return;
        }

        if token.is_end_tag() && token.tag_name() == "colgroup" {
            if self.current_node().as_element().tag_name() != "colgroup" {
                return self.unexpected(&token);
            }

            self.open_elements.pop();
            self.switch_to(InsertMode::InTable);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "col" {
            return self.unexpected(&token);
        }

        if (token.is_start_tag() || token.is_end_tag()) && token.tag_name() == "template" {
            self.handle_in_head(token);
            return;
        }

        if let Token::EOF = token {
            self.handle_in_body(token);
            return;
        }

        if self.current_node().as_element().tag_name() != "colgroup" {
            return self.unexpected(&token);
        }

        self.open_elements.pop();
        self.switch_to(InsertMode::InTable);
        return self.process(token);
    }

    fn handle_in_table_body(&mut self, token: Token) {
        if token.is_start_tag() && token.tag_name() == "tr" {
            self.open_elements.clear_back_to_table_context();
            self.insert_html_element(token);
            self.switch_to(InsertMode::InRow);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "th", "td") {
            self.unexpected(&token);
            self.open_elements.clear_back_to_table_context();
            self.insert_html_element(Token::new_start_tag_with_name("tr"));
            self.switch_to(InsertMode::InRow);
            return self.process(token);
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "tbody", "tfoot", "thead") {
            if !self
                .open_elements
                .has_element_name_in_table_scope(&token.tag_name())
            {
                self.unexpected(&token);
                return;
            }

            self.open_elements.clear_back_to_table_context();
            self.open_elements.pop();
            self.switch_to(InsertMode::InTable);
            return;
        }

        if (token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "caption",
                "col",
                "colgroup",
                "tbody",
                "tfoot",
                "thead"
            ))
            || (token.is_end_tag() && token.tag_name() == "table")
        {
            if !self.open_elements.has_element_name_in_table_scope("tbody")
                && !self.open_elements.has_element_name_in_table_scope("thead")
                && !self.open_elements.has_element_name_in_table_scope("tfoot")
            {
                self.unexpected(&token);
                return;
            }

            self.open_elements.clear_back_to_table_context();
            self.open_elements.pop();
            self.switch_to(InsertMode::InTable);
            return self.process(token);
        }

        if token.is_end_tag()
            && match_any!(
                token.tag_name(),
                "body",
                "caption",
                "col",
                "colgroup",
                "html",
                "td",
                "th",
                "tr"
            )
        {
            self.unexpected(&token);
            return;
        }

        return self.handle_in_table(token);
    }

    fn handle_in_row(&mut self, token: Token) {
        if token.is_start_tag() && match_any!(token.tag_name(), "th", "td") {
            self.open_elements.clear_back_to_table_context();
            self.insert_html_element(token);
            self.switch_to(InsertMode::InCell);
            self.active_formatting_elements.add_marker();
            return;
        }

        if token.is_end_tag() && token.tag_name() == "tr" {
            if !self.open_elements.has_element_name_in_table_scope("tr") {
                self.unexpected(&token);
                return;
            }

            self.open_elements.clear_back_to_table_context();
            self.open_elements.pop();
            self.switch_to(InsertMode::InTableBody);
            return;
        }

        if (token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "caption",
                "col",
                "colgroup",
                "tbody",
                "tfoot",
                "thead",
                "tr"
            ))
            || (token.is_end_tag() && token.tag_name() == "table")
        {
            if !self.open_elements.has_element_name_in_table_scope("tr") {
                self.unexpected(&token);
                return;
            }
            self.open_elements.clear_back_to_table_context();
            self.open_elements.pop();
            self.switch_to(InsertMode::InTableBody);
            return self.process(token);
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "tbody", "tfoot", "thead") {
            if !self
                .open_elements
                .has_element_name_in_table_scope(&token.tag_name())
            {
                self.unexpected(&token);
                return;
            }
            if !self.open_elements.has_element_name_in_table_scope("tr") {
                self.unexpected(&token);
                return;
            }
            self.open_elements.clear_back_to_table_context();
            self.open_elements.pop();
            self.switch_to(InsertMode::InTableBody);
            return self.process(token);
        }

        if token.is_end_tag()
            && match_any!(
                token.tag_name(),
                "body",
                "caption",
                "col",
                "colgroup",
                "html",
                "td",
                "th"
            )
        {
            self.unexpected(&token);
            return;
        }

        return self.handle_in_table(token);
    }

    fn handle_in_cell(&mut self, token: Token) {
        if token.is_end_tag() && match_any!(token.tag_name(), "td", "th") {
            if !self
                .open_elements
                .has_element_name_in_table_scope(&token.tag_name())
            {
                self.unexpected(&token);
                return;
            }

            self.generate_implied_end_tags("");

            if self.current_node().as_element().tag_name() != *token.tag_name() {
                emit_error!("Expected current node to have same tag name as token");
            }
            self.open_elements.pop_until(token.tag_name());
            self.active_formatting_elements.clear_up_to_last_marker();
            self.switch_to(InsertMode::InRow);
            return;
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "caption",
                "col",
                "colgroup",
                "tbody",
                "td",
                "tfoot",
                "th",
                "thead",
                "tr"
            )
        {
            if !self.open_elements.has_element_name_in_table_scope("td")
                || !self.open_elements.has_element_name_in_table_scope("th")
            {
                self.unexpected(&token);
                return;
            }

            self.close_cell();
            return self.process(token);
        }

        if token.is_end_tag()
            && match_any!(
                token.tag_name(),
                "body",
                "caption",
                "col",
                "colgroup",
                "html"
            )
        {
            self.unexpected(&token);
            return;
        }

        if token.is_end_tag()
            && match_any!(token.tag_name(), "table", "tbody", "tfoot", "thead", "tr")
        {
            if !self
                .open_elements
                .has_element_name_in_table_scope(&token.tag_name())
            {
                self.unexpected(&token);
                return;
            }
            self.close_cell();
            return self.process(token);
        }

        return self.handle_in_body(token);
    }

    fn handle_in_select(&mut self, token: Token) {
        if let Token::Character(c) = token {
            if c == '\0' {
                self.unexpected(&token);
                return;
            }
            return self.insert_character(c);
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_start_tag() && token.tag_name() == "option" {
            if self.current_node().as_element().tag_name() == "option" {
                self.open_elements.pop();
            }
            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "optgroup" {
            if self.current_node().as_element().tag_name() == "option" {
                self.open_elements.pop();
            }
            if self.current_node().as_element().tag_name() == "optgroup" {
                self.open_elements.pop();
            }
            self.insert_html_element(token);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "optgroup" {
            if self.current_node().as_element().tag_name() == "option" {
                let el = self.open_elements.get(self.open_elements.len() - 1);
                if el.as_element().tag_name() == "optgroup" {
                    self.open_elements.pop();
                }
            }
            if self.current_node().as_element().tag_name() == "optgroup" {
                self.open_elements.pop();
            } else {
                emit_error!("expected optgroup");
            }
            return;
        }

        if token.is_end_tag() && token.tag_name() == "option" {
            if self.current_node().as_element().tag_name() == "option" {
                self.open_elements.pop();
            } else {
                self.unexpected(&token);
            }
            return;
        }

        if token.is_end_tag() && token.tag_name() == "select" {
            if !self
                .open_elements
                .has_element_name_in_select_scope("select")
            {
                self.unexpected(&token);
                return;
            }
            self.open_elements.pop_until("select");
            self.reset_insertion_mode_appropriately();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "select" {
            self.unexpected(&token);
            if !self
                .open_elements
                .has_element_name_in_select_scope("select")
            {
                return;
            }
            self.open_elements.pop_until("select");
            self.reset_insertion_mode_appropriately();
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "input", "keygen", "textarea") {
            self.unexpected(&token);
            if !self
                .open_elements
                .has_element_name_in_select_scope("select")
            {
                return;
            }
            self.open_elements.pop_until("select");
            self.reset_insertion_mode_appropriately();
            return self.process(token);
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "script", "template") {
            return self.handle_in_head(token);
        }

        if token.is_end_tag() && token.tag_name() == "template" {
            return self.handle_in_head(token);
        }

        if token.is_eof() {
            return self.handle_in_body(token);
        }

        self.unexpected(&token);
        return;
    }

    fn handle_in_select_in_table(&mut self, token: Token) {
        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "caption",
                "table",
                "tbody",
                "tfoot",
                "thead",
                "tr",
                "td",
                "th"
            )
        {
            self.unexpected(&token);
            self.open_elements.pop_until("select");
            self.reset_insertion_mode_appropriately();
            return self.process(token);
        }

        if token.is_end_tag()
            && match_any!(
                token.tag_name(),
                "caption",
                "table",
                "tbody",
                "tfoot",
                "thead",
                "tr",
                "td",
                "th"
            )
        {
            self.unexpected(&token);
            if !self
                .open_elements
                .has_element_name_in_table_scope(&token.tag_name())
            {
                return;
            }
            self.open_elements.pop_until("select");
            self.reset_insertion_mode_appropriately();
            return self.process(token);
        }
        return self.handle_in_select(token);
    }

    fn handle_in_template(&mut self, token: Token) {
        if let Token::Character(_) = token {
            return self.handle_in_body(token);
        }
        if let Token::Comment(_) = token {
            return self.handle_in_body(token);
        }
        if let Token::DOCTYPE { .. } = token {
            return self.handle_in_body(token);
        }
        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "base",
                "basefont",
                "bgsound",
                "link",
                "meta",
                "noframes",
                "script",
                "style",
                "template",
                "title"
            )
        {
            return self.handle_in_head(token);
        }
        if token.is_end_tag() && token.tag_name() == "template" {
            return self.handle_in_head(token);
        }
        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "caption",
                "colgroup",
                "tbody",
                "tfoot",
                "thead"
            )
        {
            self.stack_of_template_insert_mode.pop();
            self.stack_of_template_insert_mode.push(InsertMode::InTable);
            self.switch_to(InsertMode::InTable);
            return self.process(token);
        }
        if token.is_start_tag() && token.tag_name() == "col" {
            self.stack_of_template_insert_mode.pop();
            self.stack_of_template_insert_mode
                .push(InsertMode::InColumnGroup);
            self.switch_to(InsertMode::InColumnGroup);
            return self.process(token);
        }
        if token.is_start_tag() && token.tag_name() == "tr" {
            self.stack_of_template_insert_mode.pop();
            self.stack_of_template_insert_mode
                .push(InsertMode::InTableBody);
            self.switch_to(InsertMode::InTableBody);
            return self.process(token);
        }
        if token.is_start_tag() && match_any!(token.tag_name(), "td", "th") {
            self.stack_of_template_insert_mode.pop();
            self.stack_of_template_insert_mode.push(InsertMode::InRow);
            self.switch_to(InsertMode::InRow);
            return self.process(token);
        }
        if token.is_start_tag() {
            self.stack_of_template_insert_mode.pop();
            self.stack_of_template_insert_mode.push(InsertMode::InBody);
            self.switch_to(InsertMode::InBody);
            return self.process(token);
        }
        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }
        if token.is_eof() {
            if !self.open_elements.contains("template") {
                self.stop_parsing();
                return;
            }
            self.unexpected(&token);
            self.open_elements.pop_until("template");
            self.active_formatting_elements.clear_up_to_last_marker();
            self.stack_of_template_insert_mode.pop();
            self.reset_insertion_mode_appropriately();
            return self.process(token);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn handle_initial_correctly() {
        let html = "<!-- this is a test -->";
        let tokenizer = Tokenizer::new(html.chars());
        let tree_builder = TreeBuilder::default(tokenizer);

        assert_eq!(
            tree_builder
                .run()
                .first_child()
                .unwrap()
                .as_comment()
                .get_data(),
            " this is a test "
        );
    }

    #[test]
    fn handle_parsing_children_correctly() {
        let html = "<div><div></div><div></div><div></div></div>";
        let tokenizer = Tokenizer::new(html.chars());
        let tree_builder = TreeBuilder::default(tokenizer);
        let document = tree_builder.run();

        let html = document.first_child().unwrap();
        let body = html.last_child().unwrap();
        let div = body.first_child().unwrap();

        assert_eq!(NodePtr(div).child_nodes().length(), 3);
    }

    #[test]
    fn handle_parsing_a_tag() {
        let html = "<div><a href=\"http://google.com\">This is a link</a></div>";
        let tokenizer = Tokenizer::new(html.chars());
        let tree_builder = TreeBuilder::default(tokenizer);
        let document = tree_builder.run();

        let html = document.first_child().unwrap();
        let body = html.last_child().unwrap();
        let div = body.first_child().unwrap();
        let a = div.first_child().unwrap();

        assert_eq!(
            a.as_element().attributes().borrow().get_str("href"),
            "http://google.com".to_string()
        );
        assert_eq!(NodePtr(a).child_text_content(), "This is a link".to_string());
    }
}
