use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let text = r#"
        start: expr
        expr: expr "+" INT | INT
        %import ( WS, INT )
        %ignore WS
        "#;

    let opt = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    let text_parser = Swiftlet::from_string(text, opt);

    let parsed = text_parser.parse("1234 + 3953");

    match parsed {
        Ok(ast) => ast.pretty_print(),
        Err(err) => eprintln!("{}", err),
    }
}
