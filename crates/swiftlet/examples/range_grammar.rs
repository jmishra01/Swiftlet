use std::sync::Arc;

use swiftlet::grammar::Algorithm;
use swiftlet::{Swiftlet, ParserOption};

fn main() {
    let text = r#"
        start: expr
        expr: expr "+" INT | INT
        DIGIT: "0" .. "9"
        INT: DIGIT+
        %import WS
        %ignore WS
        "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });

    let text_parser = Swiftlet::from_string(text, conf);

    match text_parser.parse("1234 + 3953") {
        Ok(res) => {
            res.pretty_print();
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
