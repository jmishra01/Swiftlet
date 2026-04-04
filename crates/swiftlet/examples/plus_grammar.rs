use std::sync::Arc;
use swiftlet::ast::Ast;
use swiftlet::{ParserConfig, Swiftlet};

fn transform(ast: &Ast) -> Option<i32> {
    match ast {
        Ast::Token(token) => token.word().parse::<i32>().ok(),
        Ast::Tree(_, children) => {
            let mut r = 0;
            for i in children.iter() {
                if let Some(n) = transform(i) {
                    r += n;
                }
            }
            Some(r)
        }
    }
}

fn main() {
    let text = r#"
        start: expr
        expr: (number "+")+ number
        number: number? DIGIT
        DIGIT: "0" .. "9"
        %import WS
        %ignore WS
        "#;

    let conf = Arc::new(ParserConfig::default());

    let text_parser = Swiftlet::from_str(text)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");

    match text_parser.parse("1 + 2 + 3") {
        Ok(res) => {
            res.pretty_print();
            println!("Result = {:#?}", transform(&res).unwrap());
        }
        Err(e) => {
            eprintln!("{:#?}", e);
        }
    }
}
