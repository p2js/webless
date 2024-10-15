use std::ops::Range;

use crate::ast::*;

/// Foreign elements; elements that are not expected to contain HTML,
/// Meaning the parser will treat their inner text as a HtmlNode::Foreign.
const FOREIGN_ELEMENTS: [&str; 6] = ["script", "style", "title", "textarea", "svg", "math"];

/// Self-closing elements; no children or matching closing tag.
const VOID_ELEMENTS: [&str; 16] = [
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
    "meta", "param", "source", "track", "wbr",
];

/// Macro to match for space characters
macro_rules! space_chars {
    () => {
        b' ' | b'\n' | b'\r' | b'\t' | b'\x0C'
    };
}

/// Macro to match for control characters besides spaces
macro_rules! control_chars {
    () => {
        b'\x00'..=b'\x08' | b'\x0B' | b'\x0E'..=b'\x1F'| b'\x7F'
    };
}

/// Helper to test that a string is in a list, ignoring ascii case
fn contains_ignore_ascii_case(list: &[&str], str: &str) -> bool {
    list.iter().any(|term| term.eq_ignore_ascii_case(str))
}

pub fn parse(source: &str) -> HTMLDocument {
    ParseString::new(source).parse()
}

/// Internal utility type representing the details of a parse error.
type InternalParseError = String;
/// Internal utility type representing result returned by node parsing functions that can fail.
type NodeResult<'a> = Result<HTMLNode<'a>, InternalParseError>;
/// Internal utility type representing result returned by attribute parsing functions that can fail.
type AttributeResult<'a> = Result<HTMLAttribute<'a>, InternalParseError>;

struct ParseString<'a> {
    source: &'a str,
    current_index: usize,
}

impl<'a> ParseString<'a> {
    fn new(source: &'a str) -> Self {
        ParseString {
            source,
            current_index: 0,
        }
    }

    fn parse(&mut self) -> HTMLDocument<'a> {
        let mut html_nodes = vec![];

        while !self.is_at_end() {
            html_nodes.push(self.strict_node().unwrap());
        }

        HTMLDocument {
            html: html_nodes.into_boxed_slice(),
        }
    }
}

// PARSING HELPERS
impl<'a> ParseString<'a> {
    /// Helper for the parser to know if it has reached the end of the string.
    fn is_at_end(&self) -> bool {
        self.current_index >= self.source.as_bytes().len()
    }

    /// Helper to advance the current index and return character
    fn advance(&mut self) {
        self.current_index += 1
    }

    /// Helper to get current character
    fn current(&self) -> Option<u8> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source.as_bytes()[self.current_index])
        }
    }

    /// Helper to convert the current character to a String, used in errors.
    fn current_as_string(&self) -> String {
        match self.current() {
            None => String::from("[document end]"),
            Some(control @ control_chars!()) => {
                format!("[control character {:#x}]", control)
            }
            Some(char) => (char as char).to_string(),
        }
    }

    /// Helper to check whether the current character is alphanumeric
    fn current_is_alphanumeric(&self) -> bool {
        matches!(
            self.current(),
            Some(b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
        )
    }

    /// Helper to provide lookahead
    fn peek(&self, offset: usize) -> Option<u8> {
        let would_be_index = self.current_index + offset;
        if would_be_index >= self.source.as_bytes().len() {
            return None;
        }
        Some(self.source.as_bytes()[would_be_index])
    }

    /// Helper that returns whether the current character matches
    fn current_matches(&self, char: u8) -> bool {
        if let Some(current) = self.current() {
            current == char
        } else {
            false
        }
    }

    /// Helper that returns whether the next characters match
    fn next_match(&self, chars: &[u8]) -> bool {
        let next_slice = self.source.get(self.current_index..);
        match next_slice {
            Some(slice) => slice.as_bytes().starts_with(chars),
            None => false,
        }
    }

    /// Helper to expect a specific character and error otherwise
    fn expect(&self, what: &str, char: u8) -> Result<(), InternalParseError> {
        if self.current_matches(char) {
            Ok(())
        } else {
            Err(format!(
                "Expected {} '{}', found '{}'",
                what,
                char as char,
                self.current_as_string()
            ))
        }
    }

    /// Helper that advances as long as it sees space characters
    fn ignore_whitespace(&mut self) {
        while let Some(space_chars!()) = self.current() {
            self.advance();
        }
    }

    /// Helper to fully consume an alphanumeric range of characters, and return the resulting range to reference in a string
    fn consume_alphanumeric(&mut self) -> Result<Range<usize>, InternalParseError> {
        if !self.current_is_alphanumeric() {
            return Err(format!(
                "Expected alphanumeric, found '{}'",
                self.current_as_string()
            ));
        }

        let starting_index = self.current_index;
        while self.current_is_alphanumeric() {
            self.advance();
        }
        Ok(starting_index..self.current_index)
    }
}

// GRAMMAR IMPLEMENTATION
impl<'a> ParseString<'a> {
    /// Function to parse any kind of HTML node other than text.
    fn strict_node(&mut self) -> NodeResult<'a> {
        self.ignore_whitespace();

        self.expect("start of a node", b'<')?;

        match self.peek(1) {
            None => Err(String::from("Expected something after start of node")),
            Some(b'!') => {
                // If there is a -, it is a comment
                if let Some(b'-') = self.peek(2) {
                    return self.comment();
                }
                // Otherwise attempt DOCTYPE
                let decl = self.doctype_declaration();

                match decl {
                    Err(_) => Err(String::from("Expected doctype declaration or comment")),
                    _ => decl,
                }
            } // doctype or comment
            _ => self.element(), // element
        }
    }

    /// Function to parse any kind of HTML node, including text.
    fn node(&mut self) -> NodeResult<'a> {
        if !self.current_matches(b'<') {
            return self.text();
        }

        self.strict_node()
    }

    /// Function to parse regular HTML elements.
    fn element(&mut self) -> NodeResult<'a> {
        // consume <
        self.advance();
        // get tag name
        let element_name = &self.source[self.consume_alphanumeric()?];

        self.ignore_whitespace();

        // parse attributes
        let mut attributes: Vec<HTMLAttribute<'a>> = vec![];

        while !self.current_matches(b'>') && !self.current_matches(b'/') {
            let attribute = self.attribute()?;

            if attributes.iter().any(|a| a.name == attribute.name) {
                return Err(String::from(
                    "Element has two attributes with the same name",
                ));
            }

            attributes.push(attribute);
            self.ignore_whitespace();
        }

        if contains_ignore_ascii_case(&VOID_ELEMENTS, element_name) {
            // Void element, tag closer may optionally have a '/'
            if self.current_matches(b'/') {
                self.advance();
            }
            // consume >
            self.expect("end of opening tag", b'>')?;
            self.advance();

            return Ok(HTMLNode::Element {
                name: element_name,
                attributes: attributes.into_boxed_slice(),
                children: Box::new([]),
            });
        }

        // Otherwise, not a node element, consume >
        self.expect("end of opening tag", b'>')?;
        self.advance();

        let mut children = vec![];
        if contains_ignore_ascii_case(&FOREIGN_ELEMENTS, element_name) {
            children.push(self.foreign_text(element_name)?);
        } else {
            while !self.next_match(b"</") {
                if self.current().is_none() || self.peek(1).is_none() {
                    return Err(format!(
                        "Expected matching closing tag for {}",
                        element_name
                    ));
                }
                children.push(self.node()?);
            }
        }

        // Consume </
        self.current_index += 2;

        // Get closing element name and ensure it maches
        let closing_tag_name = &self.source[self.consume_alphanumeric()?];

        if !closing_tag_name.eq_ignore_ascii_case(element_name) {
            return Err(format!(
                "Mismatched closing tag: Expected '{}', found '{}'",
                element_name, closing_tag_name
            ));
        }
        self.ignore_whitespace();
        // consume >
        self.expect("end of opening tag", b'>')?;
        self.advance();

        Ok(HTMLNode::Element {
            name: element_name,
            attributes: attributes.into_boxed_slice(),
            children: children.into_boxed_slice(),
        })
    }

    fn attribute(&mut self) -> AttributeResult<'a> {
        // Match for element name
        let name_start = self.current_index;
        while !matches!(
            self.current(),
            Some(space_chars!() | control_chars!() | b'"' | b'\'' | b'>' | b'/' | b'=') | None
        ) {
            self.advance();
        }
        if self.current_index - name_start == 0 {
            return Err(String::from("Expected attribute name"));
        }
        let name = &self.source[name_start..self.current_index];

        if let Some(control_chars!()) = self.current() {
            return Err(format!(
                "Unexpected control character {}",
                self.current_as_string()
            ));
        }

        self.ignore_whitespace();
        if self.current().is_none() {
            return Err(String::from("Expected something after attribute name"));
        }

        let mut value = "";
        if self.current_matches(b'=') {
            // consume =
            self.advance();
            match self.current() {
                None => return Err(String::from("Expected attribute value after =")),
                Some(quote @ (b'\'' | b'"')) => {
                    // Quoted attribute-value syntax
                    // consume opening quote
                    self.advance();
                    let value_start = self.current_index;
                    while !matches!(self.current(), Some(control_chars!()) | None)
                        && self.current() != Some(quote)
                    {
                        self.advance();
                    }
                    self.expect("value-ending quote", quote)?;

                    value = &self.source[value_start..self.current_index];

                    // consume closing quote
                    self.advance();
                }
                Some(_) => {
                    // Unquoted attribute-value syntax
                    let value_start = self.current_index;
                    while !matches!(
                        self.current(),
                        Some(
                            control_chars!()
                                | space_chars!()
                                | b'"'
                                | b'\''
                                | b'='
                                | b'>'
                                | b'<'
                                | b'`'
                        ) | None,
                    ) {
                        self.advance();
                    }
                    value = &self.source[value_start..self.current_index];
                }
            }
        }

        Ok(HTMLAttribute { name, value })
    }

    /// Function to parse text nodes inside elements
    fn text(&mut self) -> NodeResult<'a> {
        let starting_index = self.current_index;
        while !matches!(self.current(), Some(control_chars!() | b'<') | None) {
            self.advance();
        }
        Ok(HTMLNode::Text(
            &self.source[starting_index..self.current_index],
        ))
    }

    /// Function to parse foreign text, which will continue until it sees </element_name
    fn foreign_text(&mut self, element_name: &str) -> NodeResult<'a> {
        let starting_index = self.current_index;

        while self.current().is_some() {
            // Verify any </ encountered is not the closing tag
            if self.next_match(b"</") {
                // get the next (element_name length) characters after </
                let offset = self.current_index + 2;
                let next_chars = self
                    .source
                    .get(offset..(offset + element_name.len()))
                    .unwrap_or("");
                // break if the slice equals the name
                if next_chars.eq_ignore_ascii_case(element_name) {
                    break;
                }
            }
            // Otherwise consume
            self.advance();
        }
        if self.current().is_none() {
            return Err(format!("Expected closing tag </{element_name}>"));
        }
        // Return Foreign node with slice
        return Ok(HTMLNode::Foreign(
            &self.source[starting_index..self.current_index],
        ));
    }

    /// Function to parse a comment
    fn comment(&mut self) -> NodeResult<'a> {
        // Consume <!--
        self.current_index += 3;
        self.expect("second - in comment declaration", b'-')?;
        self.advance();

        let starting_index = self.current_index;

        if self.next_match(b"->") || self.current_matches(b'-') {
            return Err(String::from("Comments may not start with '>' or '->'"));
        }

        while self.current().is_some() {
            // If there is a --, check that the following character is >
            if self.next_match(b"--") {
                if self.peek(2) == Some(b'>') {
                    break;
                } else {
                    return Err(String::from("Comments may not contain '--'"));
                }
            }
            // Otherwise consume
            self.advance();
        }
        if self.current().is_none() {
            return Err(String::from("Expected comment tag closer '-->'"));
        }
        let comment_text = &self.source[starting_index..self.current_index];
        // Consume -->
        self.current_index += 3;

        Ok(HTMLNode::Comment(comment_text))
    }

    /// Function to parse a DOCYPE declaration
    fn doctype_declaration(&mut self) -> NodeResult<'a> {
        // Check that DOCTYPE follows <!
        if !self.source[(self.current_index + 2)..(self.current_index + 9)]
            .eq_ignore_ascii_case("DOCTYPE")
        {
            return Err(String::new());
        }
        // Consume <!DOCTYPE
        self.current_index += 9;
        self.ignore_whitespace();

        // This parser does not concern itself with actually parsing doctypes.
        // Feel free to add it and open a PR if you'd like.
        let starting_index = self.current_index;
        while !matches!(self.current(), Some(b'>') | None) {
            self.advance();
        }
        if self.current().is_none() {
            return Err(String::from("Expected DOCTYPE tag closer '>'"));
        }
        let doctype_string = &self.source[starting_index..self.current_index];
        // Consume >
        self.advance();

        Ok(HTMLNode::Doctype(doctype_string))
    }
}
