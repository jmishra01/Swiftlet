use std::sync::Arc;
use swiftlet::{Swiftlet, ParserOption};
use swiftlet::grammar::Algorithm;


fn main() {

    let grammar = r#"
    start: "select" "selectbar"
    "#;

    let po = Arc::new(
        ParserOption {
            debug: true,
            algorithm: Algorithm::CLR,
            ..Default::default()});
    let parser = Swiftlet::from_string(grammar, po.clone()).expect("failed to build parser");

    let parsed = parser.parse("selectselectbar");

    parsed.unwrap().pretty_print();
}
