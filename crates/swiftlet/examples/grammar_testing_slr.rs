use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    // Below grammar is not LR(0), as it contain SR conflicts
    let text = r#"
        a: b
        b: b c | c
        c: e | d
        d: "A"
        e: "B"
        %import WS
        %ignore WS
        "#;
    let conf = Arc::new(ParserOption {
        start: "a".to_string(),
        ..Default::default()
    });
    let text_parser = Swiftlet::from_string(text, conf);
    let ast = text_parser.parse("AB");
    ast.unwrap().print();
    // LR(0)  -> Failed to parse
    // SLR    -> a([b([b([c([d([A])])]), c([e([B])])])])
    // Earley -> a([b([b([c([d([A])])]), c([e([B])])])])
}
