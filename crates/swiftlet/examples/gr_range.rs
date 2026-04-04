use std::sync::Arc;
use std::time::Instant;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let t1 = Instant::now();
    let text = r#"
        start: expr
        expr: expr "+" NUMBER | NUMBER
        NUMBER: DIGIT+
        DIGIT: "0" .. "9"
        %import WS
        %ignore WS
        "#;

    let conf = Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });

    let text_parser = Swiftlet::from_str(text)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    if let Ok(_res) = text_parser.parse("123 + 234 + 456") {
        _res.pretty_print();
    }
    println!("Time took: {:?}", t1.elapsed());
}
