#![cfg(test)]

use super::*;

#[test]
fn parse_simple_element() {
    dbg!(parse("<html><div><hr/><hr></div></html>"));
}
