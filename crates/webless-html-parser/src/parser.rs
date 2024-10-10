use std::ops::Range;

use crate::ast::*;

/*
Grammar definition:

document    -> strictNode*;

strictNode  -> doctype | comment | voidElement | foreign | element
node        -> strictNode | textNode

comment     -> "<!--" TEXT "-->"
doctype     -> "<!DOCTYPE" TYPE ">"

voidElement -> "<" VOID_ELEMENT_NAME ((attribute)*)? ("/")? ">"
foreign     -> "<" FOREIGN_ELEMENT_NAME ((attribute)*)? ">" TEXT "</" FOREIGN_ELEMENT_NAME ">"
element     -> "<" NAME ((attribute)*)? ("/")? ">" (node)* "</" NAME ">"

attribute   -> KEY ("=" (NON_QUOTED_VALUE | QUOTED_VALUE))?

textNode    -> TEXT
*/

/// Foreign elements; elements that are not expected to contain HTML,
/// Meaning the parser will treat their inner text as a HtmlNode::Foreign.
const FOREIGN_ELEMENTS: [&str; 4] = ["script", "style", "svg", "math"];

/// Self-closing elements; no children or matching closing tag.
const VOID_ELEMENTS: [&str; 16] = [
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
    "meta", "param", "source", "track", "wbr",
];

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
    ///Helper for the parser to know if it has reached the end of the string.
    fn is_at_end(&self) -> bool {
        self.current_index >= self.source.as_bytes().len()
    }

    ///Helper to advance the current index and return character
    fn advance(&mut self) {
        self.current_index += 1
    }

    ///Helper to get current character
    fn current(&self) -> Option<u8> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source.as_bytes()[self.current_index])
        }
    }

    /// Helper to convert the current character to a String, used in errors.
    fn current_as_string(&self) -> String {
        if let Some(c) = self.current() {
            (c as char).to_string()
        } else {
            String::from("[document end]")
        }
    }

    ///Helper to check whether the current character is alphanumeric
    fn current_is_alphanumeric(&self) -> bool {
        matches!(
            self.current(),
            Some(b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
        )
    }

    ///Helper to return previous character
    fn previous(&self) -> Option<u8> {
        if self.current_index == 0 {
            None
        } else {
            Some(self.source.as_bytes()[self.current_index - 1])
        }
    }

    ///Helper to provide lookahead
    fn peek(&self, offset: usize) -> Option<u8> {
        let would_be_index = self.current_index + offset;
        if would_be_index >= self.source.as_bytes().len() {
            return None;
        }
        Some(self.source.as_bytes()[would_be_index])
    }

    ///Helper that returns whether the current character matches
    fn matches(&self, char: u8) -> bool {
        if let Some(current) = self.current() {
            current == char
        } else {
            false
        }
    }

    ///Helper to expect a specific character and error otherwise
    fn expect(&self, char: u8, what: &str) -> Result<(), InternalParseError> {
        if self.matches(char) {
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

    ///Helper that advances as long as it sees white space
    fn ignore_whitespace(&mut self) {
        while let Some(b' ') | Some(b'\n') | Some(b'\r') | Some(b'\t') = self.current() {
            self.advance();
        }
    }

    ///Helper to fully consume an alphanumeric range of characters, and return the resulting range to reference in a string
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

    ///Helper to handle and consume a tag closer (> with optional preceding ' /')
    fn consume_tag_closer(&mut self) -> Result<(), InternalParseError> {
        // Consume optional /
        if self.matches(b'/') {
            if self.previous().unwrap() != b' ' {
                return Err(String::from("Expected space before '/'"));
            }
            self.advance();
        }
        //consume >
        self.expect(b'>', "end of opening tag")?;
        self.advance();
        Ok(())
    }
}

// GRAMMAR IMPLEMENTATION
impl<'a> ParseString<'a> {
    /// Function to parse any kind of HTML node other than text.
    fn strict_node(&mut self) -> NodeResult<'a> {
        self.ignore_whitespace();

        self.expect(b'<', "start of a node")?;

        match self.peek(1) {
            None => Err(String::from("Expected something after start of node")),
            Some(b'!') => todo!("DOCTYPE declarations and comments"), // doctype or comment
            _ => self.element(),                                      // element
        }
    }

    /// Function to parse any kind of HTML node, including text.
    fn node(&mut self) -> NodeResult<'a> {
        if !self.matches(b'<') {
            return self.text();
        }

        self.strict_node()
    }

    ///Function to parse regular HTML elements.
    fn element(&mut self) -> NodeResult<'a> {
        //consume <
        self.advance();
        //get tag name
        let element_name = &self.source[self.consume_alphanumeric()?];

        self.ignore_whitespace();

        //parse attributes
        let mut attributes = vec![];

        while !self.matches(b'>') && !self.matches(b'/') {
            attributes.push(self.attribute()?);
            self.ignore_whitespace();
        }

        self.consume_tag_closer()?;

        if VOID_ELEMENTS.contains(&element_name) {
            // Void element, return just that
            return Ok(HTMLNode::Element {
                name: element_name,
                attributes: attributes.into_boxed_slice(),
                children: Box::new([]),
            });
        }

        let mut children = vec![];
        if FOREIGN_ELEMENTS.contains(&element_name) {
            children.push(self.foreign_text()?);
        } else {
            while self.current() != Some(b'<') || self.peek(1) != Some(b'/') {
                if self.current().is_none() || self.peek(1).is_none() {
                    return Err(format!(
                        "Expected matching closing tag for {}",
                        element_name
                    ));
                }
                children.push(self.node()?);
            }
        }

        //Consume </
        self.advance();
        self.advance();

        //Get closing element name and ensure it maches
        let closing_tag_name = &self.source[self.consume_alphanumeric()?];

        if !closing_tag_name.eq_ignore_ascii_case(element_name) {
            return Err(format!(
                "Mismatched closing tag: Expected '{}', found '{}'",
                element_name, closing_tag_name
            ));
        }
        self.ignore_whitespace();
        self.consume_tag_closer()?;

        Ok(HTMLNode::Element {
            name: element_name,
            attributes: attributes.into_boxed_slice(),
            children: children.into_boxed_slice(),
        })
    }

    fn attribute(&mut self) -> AttributeResult<'a> {
        todo!("element attributes")
    }

    fn text(&mut self) -> NodeResult<'a> {
        todo!("Text node children")
    }

    fn foreign_text(&mut self) -> NodeResult<'a> {
        todo!("Foreign text")
    }
}
