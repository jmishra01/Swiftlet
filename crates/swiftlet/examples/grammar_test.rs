use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
    start: expr
    expr: "1"
    %import WS
    %ignore WS
    "#;

    let conf = Arc::new(ParserOption::default());

    let parser = Swiftlet::from_string(grammar, conf);
    let text = "1";
    if let Ok(parsed) = parser.parse(text) {
        parsed.pretty_print();
    }
}
