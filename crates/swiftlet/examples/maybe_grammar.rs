use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let text = r#"
        start: expr
        expr: A ["-" "+"] B
        A: "A"
        B: "B"
        %import WS
        %import INT
        %ignore WS
        "#;

    let conf = Arc::new(ParserOption::default());
    let text_parser = Swiftlet::from_string(text, conf).expect("failed to build parser");

    for w in ["AB", "A-+B"].iter() {
        match text_parser.parse(w) {
            Ok(res) => res.pretty_print(),
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    }
}
