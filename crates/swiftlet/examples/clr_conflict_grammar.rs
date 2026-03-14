use swiftlet::{Ambiguity, Swiftlet, ParserOption};
use std::sync::Arc;
fn main() {
    let grammar = r#"
    s: e
    e: e? t
    t: t? DIGIT
    DIGIT: "0" .. "9"
    "#;

    let conf = Arc::new(ParserOption {
        start: "s".to_string(),
        ambiguity: Ambiguity::Explicit,
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    match parser.parse("12") {
        Ok(ast) => ast.pretty_print(),
        Err(e) => panic!("{}", e),
    }
}
