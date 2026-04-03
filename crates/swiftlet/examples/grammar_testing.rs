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
    let text = r#"
    s: e b
    e: "A" b
    b: "B"
    "#;

    let text = r#"
    s: a+ -> x
    a: "a"
    "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::Earley,
        debug: true,
        start: "s".to_string(),
        ..Default::default()
    });
    let text_parser = Swiftlet::from_string(text, conf).expect("failed to build parser");
    // let ast = text_parser.parse("ABBABBBAB");
    // let ast = text_parser.parse("ABB");
    let ast = text_parser.parse("aaa");
    ast.unwrap().pretty_print();
}
