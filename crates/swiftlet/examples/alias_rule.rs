use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammar = r#"
        start: expr
        expr: expr "+" INT -> add
            | expr "-" INT -> sub
            | INT
        %import (WS, INT)
        %ignore WS
        "#;

    let text = "3 + 10 - 5 + 20";

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::Earley,
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    let parsed_text = parser.parse(&text);

    match parsed_text {
        Ok(tree) => {
            println!("AST");
            tree.pretty_print();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
