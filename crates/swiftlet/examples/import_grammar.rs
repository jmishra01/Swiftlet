use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let text = r#"
        start: expr
        expr: expr "+" INT | INT
        %import ( WS, INT )
        %ignore WS
        "#;

    let opt = Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    let text_parser = Swiftlet::from_str(text)
        .map(|grammar| grammar.parser(opt))
        .expect("failed to build parser");

    let parsed = text_parser.parse("1234 + 3953");

    match parsed {
        Ok(ast) => ast.pretty_print(),
        Err(err) => eprintln!("{}", err),
    }
}
