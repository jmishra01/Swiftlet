use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};
fn main() {
    let text = r#"
        e: e "+" t | t
        ?t: t "*" f | f
        f: "(" e ")" | "id"
        %import WS
        %ignore WS
        "#;
    let conf = Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        start: "e".to_string(),
        ..Default::default()
    });
    let text_parser = Swiftlet::from_str(text)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    let ast = text_parser.parse("id*id+id");
    ast.unwrap().pretty_print();
}
