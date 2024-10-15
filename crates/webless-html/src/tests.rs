#![cfg(test)]
/// This test module is only being used to check that output for parsing things
/// looks right as features are implemented.
/// Upset that there are no actual tests? feel free to write some and contribute!
use super::*;

#[test]
fn parse_example_doc() {
    dbg!(
    parse(
        r#"<!DOCTYPE html><!--Comment--><html lang='en'><h1>Hello World</h1><hr color="red"><p>Text here</p><hr bold="yes" italic/><script>javascript here </bla></script></html>"#
    ).unwrap().html()
    );
}
