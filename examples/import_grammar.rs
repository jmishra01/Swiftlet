use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
use std::sync::Arc;

fn main() {
    let text = r#"
        start: expr
        expr: expr "+" INT | INT
        %import ( WS, INT )
        %ignore WS
        "#.to_string();

    let opt = Arc::new(ParserOption { algorithm: Algorithm::CLR, ..Default::default() });
    let mut text_parser = Barat::from_string(text, opt);

    let parsed = text_parser.parse("1234 + 3953");

    match parsed {
        Ok(ast) => ast.pretty_print(),
        Err(err) => eprintln!("{}", err),
    }
}
