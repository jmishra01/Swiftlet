use std::sync::Arc;
use swiftlet::preclude::*;

fn calculate(ast: &Ast) -> i32 {
    match ast {
        Ast::Token(token) => token.word().parse::<i32>().unwrap(),
        Ast::Tree(tree, children) => match tree.as_str() {
            "start" | "expr" => calculate(&children[0]),
            "add" => calculate(&children[0]) + calculate(&children[2]),
            "sub" => calculate(&children[0]) - calculate(&children[2]),
            _ => {
                panic!("Invalid tree: {}", tree);
            }
        },
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
        "#;

    let conf = Arc::new(ParserConfig {
        algorithm: Algorithm::Earley,
        debug: true,
        ..Default::default()
    });
    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to get parser");
    let text = "10 - 2 + 5 - 2";

    match parser.parse(text) {
        Ok(tree) => {
            println!("AST: ");
            tree.pretty_print();
            println!("Total: {}", calculate(&tree));
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
