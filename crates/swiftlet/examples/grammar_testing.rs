use swiftlet::grammar::Algorithm;
use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;
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
    let text_parser = Swiftlet::from_string(text, conf);
    let ast = text_parser.parse("AB");
    ast.unwrap().print();
}
