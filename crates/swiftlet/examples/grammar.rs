use swiftlet::grammar::Algorithm;
use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let t1 = Instant::now();
    let text = r#"
        start: expr
        expr: expr "+" number | number
        number: number DIGIT | DIGIT
        DIGIT: "0" .. "9"
        %import WS
        %ignore WS
        "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });

    let text_parser = Swiftlet::from_string(text, conf);
    if let Ok(_res) = text_parser.parse("1 + 2 + 4") {
        _res.pretty_print();
    }
    println!("Time took: {:?}", t1.elapsed());
}
