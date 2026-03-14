use swiftlet::grammar::Algorithm;
use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;
fn main() {
    let text = r#"
        e: e "+" t | t
        ?t: t "*" f | f
        f: "(" e ")" | "id"
        %import WS
        %ignore WS
        "#;
    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "e".to_string(),
        ..Default::default()
    });
    let text_parser = Swiftlet::from_string(text, conf);
    let ast = text_parser.parse("id*id+id");
    ast.unwrap().pretty_print();
}
