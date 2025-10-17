use barat::{Ambiguity, Barat, ParserOption};
use std::sync::Arc;
fn main() {
    let grammar = r#"
    s: e
    e: e? t
    t: t? DIGIT
    DIGIT: "0" .. "9"
    "#
        .to_string();

    let conf = Arc::new(ParserOption {
        start: "s".to_string(),
        ambiguity: Ambiguity::Explicit,
        ..Default::default()
    });
    let mut parser = Barat::from_string(grammar, conf);
    match parser.parse("12") {
        Ok(ast) => ast.pretty_print(),
        Err(e) => panic!("{}", e),
    }
}
