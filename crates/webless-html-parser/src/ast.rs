pub struct HTMLDocument<'a> {
    html: HTMLNode<'a>,
}

pub enum HTMLNode<'a> {
    Comment(&'a str),
    TextNode(&'a str),
    Element {
        name: &'a str,
        attributes: Box<[HTMLAttribute<'a>]>,
        children: Box<[HTMLNode<'a>]>,
    },
}

pub struct HTMLAttribute<'a> {
    pub(crate) key: &'a str,
    pub(crate) val: &'a str,
}
