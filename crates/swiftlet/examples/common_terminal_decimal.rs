use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet, grammar::Algorithm};

fn main() {
    let grammar = r#"
    start: expr
    expr: expr "+" number | number
    number: DECIMAL | INT
    %import (INT, WS, DECIMAL)
    %ignore WS
    "#;

    let text = "12.34 + 10";

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        debug: true,
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", [Tree("hello", ["hello"]), Tree("world", ["world"])])
        }
        Err(err) => eprintln!("{}", err),
    }
}
