use barat::{Barat, ParserOption, lexer::AST};
use std::sync::Arc;


fn calculate(ast: &AST) -> i32 {
    match ast {
        AST::Token(token) => {
            token.word.parse::<i32>().unwrap()
        }
        AST::Tree(tree, children) => {
            match tree.as_str() {
                "start" | "expr" => calculate(&children[0]),
                "add" => calculate(&children[0]) + calculate(&children[2]),
                "sub" => calculate(&children[0]) - calculate(&children[2]),
                _ => {
                    panic!("Invalid tree: {}", tree);
                }
            }
        }
    }
}

fn main() {
    let grammar = r#"
        start: expr
        expr: expr "+" INT -> add
            | expr "-" INT -> sub
            | INT
        %import (WS, INT)
        %ignore WS
        "#
        .to_string();

    let conf = Arc::new(ParserOption::default());
    let mut parser = Barat::from_string(grammar, conf);
    let text = "10 - 2 + 5 - 2";

    match parser.parse(text) {
        Ok(tree) => {
            print!("AST: "); tree.print();
            println!("Total: {}", calculate(&tree));
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
