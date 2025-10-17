use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
        start: expr
        expr: expr "+" INT -> add
            | expr "-" INT -> sub
            | INT
        %import (WS, INT)
        %ignore WS
        "#
        .to_string();

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        debug: true,
        ..Default::default()
    });
    let mut parser = Barat::from_string(grammar, conf);
    let text = "1 + 2 - 3";
    if let Ok(_parsed) = parser.parse(text) {
        _parsed.pretty_print();
    }
}
