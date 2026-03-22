use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammar = r#"
        s: e
        e: e "+" t | t
        t: "(" e ")" | n
        n: n D | D
        D: "0" .. "9"
        %import WS
        %ignore WS
        "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });

    let parser = Swiftlet::from_string(grammar, conf);
    let text = "(1+(4+3)+4+5)";
    if let Ok(parsed) = parser.parse(text) {
        // #[cfg(feature = "debug")]
        parsed.print();
    }
}
