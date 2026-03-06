use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
use std::sync::Arc;

fn main() {
    let text = r#"
        e: t
        ?t: x | y | i
        ?x.1: t "+" t
        ?y.2: t "-" t
        i.3: "id"
        %import WS
        %ignore WS
        "#.to_string();

    let conf = Arc::new(ParserOption { start: "e".to_string(), algorithm: Algorithm::CLR, ..Default::default() });

    let mut text_parser = Barat::from_string(text, conf);

    match text_parser.parse("id + id - id") {
        Ok(res) => res.pretty_print(),
        Err(e) => { eprintln!("{:?}", e) }
    }
}