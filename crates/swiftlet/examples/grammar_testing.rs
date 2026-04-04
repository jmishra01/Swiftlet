use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};
fn main() {
    let grammar = r#"
        a: b
        b: b c | c
        c: "A" c | "B"
        %import WS
        %ignore WS
        "#;
    let grammar = r#"
    s: e b
    e: "A" b
    b: "B"
    "#;

    let grammar = r#"
    s: a+ -> x
    a: "a"
    "#;

    let grammar = r#"
    s: "A" _NL "B"
    _NL: /(\r?\n)+/
    "#;

    let texts = [r#"A
B"#, "A\nB"];

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::Earley,
        debug: true,
        start: "s".to_string(),
        ..Default::default()
    });
    let text_parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    for text in texts {
        let ast = text_parser.parse(text);
        ast.unwrap().pretty_print();
    }
}
