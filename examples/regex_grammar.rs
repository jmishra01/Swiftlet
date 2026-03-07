use std::sync::Arc;

use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};

fn main() {
    let text = r#"
        start: expr
        expr: expr "+" _v | _v
        _v: /[\w\d]+/
        %import WS
        %ignore WS
        "#.to_string();

    let conf = Arc::new(ParserOption { algorithm: Algorithm::CLR, debug: false, ..Default::default() });

    let mut text_parser = Barat::from_string(text, conf);

    match text_parser.parse("abc123 + 456efg") {
        Ok(res) => {
            res.pretty_print();
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
