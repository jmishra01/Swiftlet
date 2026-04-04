use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let text = r#"
        start: expr
        expr: expr M | N
        M: "A"
        N: "B"
        O: "C"
        %import (WS, INT)
        %ignore WS
        "#;
    let conf = Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    let text_parser = Swiftlet::from_str(text)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    match text_parser.parse("BCA") {
        Ok(res) => res.pretty_print(),
        Err(e) => {
            eprintln!("{:?}", e)
        }
    }
}
