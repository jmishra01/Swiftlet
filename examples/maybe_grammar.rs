use barat::{Barat, ParserOption};
use std::sync::Arc;

fn main() {
    let text = r#"
        start: expr
        expr: A ["-" "+"] B
        A: "A"
        B: "B"
        %import WS
        %import INT
        %ignore WS
        "#.to_string();

    let conf = Arc::new(ParserOption::default());
    let mut text_parser = Barat::from_string(text, conf);

    for w in ["AB", "A-+B"].iter() {
        match text_parser.parse(w) {
            Ok(res) => res.pretty_print(),
            Err(e) => { eprintln!("{:?}", e) }
        }
    }
}
