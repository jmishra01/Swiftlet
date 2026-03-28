use std::sync::Arc;
use swiftlet::{Ambiguity, ParserOption, Swiftlet};
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
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    match parser.parse("12") {
        Ok(ast) => ast.pretty_print(),
        Err(e) => panic!("{}", e),
    }
}
