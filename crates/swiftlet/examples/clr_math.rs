use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};
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

    let conf = Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });

    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    let text = "(1+(4+3)+4+5)";
    match parser.parse(text) {
        Ok(res) => res.pretty_print(),
        Err(e) => {
            eprintln!("{:?}", e)
        }
    }
}
