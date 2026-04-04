use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let grammar = r#"
    start: "select" "selectbar"
    "#;

    let po = Arc::new(ParserConfig {
        debug: true,
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(po.clone()))
        .expect("failed to build parser");

    let parsed = parser.parse("selectselectbar");

    parsed.unwrap().pretty_print();
}
