use barat::{Barat, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
    start: expr
    expr: "1"
    %import WS
    %ignore WS
    "#
        .to_string();

    let conf = Arc::new(ParserOption::default());

    let mut parser = Barat::from_string(grammar, conf);
    let text = "1";
    if let Ok(parsed) = parser.parse(text) {
        // println!("{text}");
        parsed.pretty_print();
    }
}
