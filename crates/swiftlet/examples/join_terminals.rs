use std::sync::Arc;

use swiftlet::preclude::*;

fn main() {
    // let g = r#"
    // s: C
    // A: D "a" "b"
    // C: "c" A
    // D: "d"
    // "#;

    // let g = r#"
    // s: A "fe"
    // A: ("1" "2") C*
    // C: /\w+/ /\d+/
    // "#;

    let g = r#"
    s: A
    A: (X | "a")+ B
    B: "b"
    X: "x"
    "#;

    let parser_opt = Arc::new(ParserOption{start: "s".to_string(), debug: true, ..Default::default() });
    let parser = Swiftlet::from_string(g, parser_opt).expect("failed to build parser");

    let parsed = parser.parse("xaxb").expect("Failed to parse a text");
    parsed.pretty_print();
}