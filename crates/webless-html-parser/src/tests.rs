#![cfg(test)]

use super::*;

#[test]
fn parse_simple_element() {
    dbg!(parse(
        r#"<html lang='en'><hr bold><hr bold="yes" italic='no'/></html>"#
    ));
}
