use std::sync::Arc;

use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let text = r#"
        start: expr
        expr: expr "+" B | B
        A: "0" .. "9"
        B: A+
        %import WS
        %ignore WS
        "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        debug: true,
        ..Default::default()
    });

    let text_parser = Swiftlet::from_string(text, conf).expect("failed to build parser");

    match text_parser.parse("1234 + 3953") {
        Ok(res) => {
            res.pretty_print();
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
