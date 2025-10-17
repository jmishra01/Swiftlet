use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
use std::sync::Arc;
fn main() {
    let text = r#"
        a: b
        b: b c | c
        c: "A" c | "B"
        %import WS
        %ignore WS
        "#.to_string();
    let conf = Arc::new(ParserOption { algorithm: Algorithm::CLR, start: "a".to_string(), ..Default::default() });
    let mut text_parser = Barat::from_string(text, conf);
    let ast = text_parser.parse("AB");
    ast.unwrap().print();
}
