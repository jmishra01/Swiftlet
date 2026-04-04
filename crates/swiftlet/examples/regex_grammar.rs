use std::sync::Arc;

use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let text = r#"
        start: expr
        expr: expr "+" _v | _v
        _v: /\w+-\d+/
        %import WS
        %ignore WS
        "#;

    let conf = Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        debug: false,
        ..Default::default()
    });

    let text_parser = Swiftlet::from_str(text)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");

    match text_parser.parse("abc-123 + efg-456") {
        Ok(res) => {
            res.pretty_print();
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
