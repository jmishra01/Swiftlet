use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};
fn main() {
    let text = r#"
        a: b
        b: b c | c
        c: "A" c | "B"
        %import WS
        %ignore WS
        "#;
    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "a".to_string(),
        ..Default::default()
    });
    let text_parser = Swiftlet::from_string(text, conf).expect("failed to build parser");
    let ast = text_parser.parse("ABBABBBAB");
    ast.unwrap().pretty_print();
}
