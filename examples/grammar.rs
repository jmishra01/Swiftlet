use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
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
        "#
        .to_string();

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });

    let mut text_parser = Barat::from_string(text, conf);
    if let Ok(_res) = text_parser.parse("1 + 2 + 4") {
        #[cfg(feature = "debug")]
        _res.pretty_print();
    }
    let t2 = Instant::now();
    println!("Time took: {:?}", t2 - t1);
}
