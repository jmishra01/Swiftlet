use barat::lexer::AST;
use barat::{Barat, ParserOption};
use std::sync::Arc;

fn transform(ast: &AST) -> Option<i32> {
    match ast {
        AST::Token(token) => token.word.parse::<i32>().ok(),
        AST::Tree(_, children) => {
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
        "#
        .to_string();

    let conf = Arc::new(ParserOption::default());

    let mut text_parser = Barat::from_string(text, conf);

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
