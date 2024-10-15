#[derive(Debug)]

pub struct HTMLDocument<'a> {
    html: Box<[HTMLNode<'a>]>,
}

impl<'a> HTMLDocument<'a> {
    pub(crate) fn new(html: Box<[HTMLNode<'a>]>) -> Self {
        Self { html }
    }
    pub fn html(&self) -> &[HTMLNode<'a>] {
        &self.html
    }
}

pub enum DocumentMode {
    Quirks,
    Standards,
}

#[derive(Debug)]
pub enum HTMLNode<'a> {
    ///Foreign text, ie. stuff inside XML, JS or CSS nodes, ignored by the parser but different from a regular text node
    Foreign(&'a str),
    ///Doctype declaration: <!DOCTYPE ...>
    Doctype(&'a str),
    ///HTML Comments: <!--This is a comment-->
    Comment(&'a str),
    ///Regular text
    Text(&'a str),
    ///All other elements
    Element {
        name: &'a str,
        attributes: Box<[HTMLAttribute<'a>]>,
        children: Box<[HTMLNode<'a>]>,
    },
}

#[derive(Debug)]
pub struct HTMLAttribute<'a> {
    pub name: &'a str,
    pub value: &'a str,
}
