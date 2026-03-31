use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammar = r#"
    start: "select" "selectbar"
    "#;

    let po = Arc::new(ParserOption {
        debug: true,
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, po.clone()).expect("failed to build parser");

    let parsed = parser.parse("selectselectbar");

    parsed.unwrap().pretty_print();
}
