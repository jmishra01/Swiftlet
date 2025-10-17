use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
        s: e
        e: e "+" t | t
        t: "(" e ")" | n
        n: n D | D
        D: "0" .. "9"
        %import WS
        %ignore WS
        "#
        .to_string();

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });

    let mut parser = Barat::from_string(grammar, conf);
    let text = "(1+(4+3)+4+5)";
    if let Ok(parsed) = parser.parse(text) {
        // #[cfg(feature = "debug")]
        parsed.print();
    }
}
