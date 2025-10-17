use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
use std::sync::Arc;
fn main() {
    let text = r#"
        e: e "+" t | t
        ?t: t "*" f | f
        f: "(" e ")" | "id"
        %import WS
        %ignore WS
        "#
        .to_string();
    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "e".to_string(),
        ..Default::default()
    });
    let mut text_parser = Barat::from_string(text, conf);
    let ast = text_parser.parse("id*id+id");
    ast.unwrap().pretty_print();
}
