use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
use std::sync::Arc;

fn main() {
    let text = r#"
        start: expr
        expr: expr M | N
        M: "A"
        N: "B"
        O: "C"
        %import (WS, INT)
        %ignore WS
        "#.to_string();
    let conf = Arc::new(ParserOption { algorithm: Algorithm::CLR, ..Default::default() });
    let mut text_parser = Barat::from_string(text, conf);
    match text_parser.parse("BCA") {
        Ok(res) => res.pretty_print(),
        Err(e) => { eprintln!("{:?}", e) }
    }
}
