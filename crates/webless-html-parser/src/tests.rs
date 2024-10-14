#![cfg(test)]
/// I am only using this test file to check that output for parsing things looks right as i go.
/// Upset that there are no actual tests? feel free to write some and contribute!
use super::*;

#[test]
fn parse_simple_element() {
    dbg!(parse(
        r#"<html lang='en'><h1>Hello World</h1><hr color="red"><p>Text here</p><hr bold="yes" italic/><script>javascript here </bla></script></html>"#
    ).html);
}
