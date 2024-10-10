#[derive(Debug)]
pub struct HTMLDocument<'a> {
    pub html: Box<[HTMLNode<'a>]>,
}

pub enum DocumentMode {
    Quirks,
    Standards,
}

#[derive(Debug)]
pub enum HTMLNode<'a> {
    ///Foreign text, ie. stuff inside XML, JS or CSS nodes, ignored by the parser but different from a regular text node
    Foreign(&'a str),
    ///Doctype declaration: <!DOCTYPE html>
    Doctype(&'a str),
    ///HTML Comments: <!--This is a comment-->
    Comment(&'a str),
    ///Regular text
    TextNode(&'a str),
    ///All other elements
    Element {
        name: &'a str,
        attributes: Box<[HTMLAttribute<'a>]>,
        children: Box<[HTMLNode<'a>]>,
    },
}

#[derive(Debug)]
pub struct HTMLAttribute<'a> {
    pub(crate) key: &'a str,
    pub(crate) val: &'a str,
}
