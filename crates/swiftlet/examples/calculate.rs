use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet, lexer::AST};

fn calculate(ast: &AST) -> i64 {
    match ast {
        AST::Token(token) => token.word().parse::<i64>().unwrap(),
        AST::Tree(rule, children) => match rule.as_str() {
            "start" | "expr" | "term" => calculate(&children[0]),
            "add" => calculate(&children[0]) + calculate(&children[2]),
            "sub" => calculate(&children[0]) - calculate(&children[2]),
            "mul" => calculate(&children[0]) * calculate(&children[2]),
            "div" => calculate(&children[0]) / calculate(&children[2]),
            _ => panic!("{} Rule not found in the AST tree", rule),
        },
    }
}

fn main() {
    let grammar = r#"
        start: expr
        expr: expr "+" term -> add
            | expr "-" term -> sub
            | term
        term: term "*" INT -> mul
            | term "/" INT -> div
            | INT
        %import (WS, INT)
        %ignore WS
        "#;

    let text = "10 * 5 - 8 / 2 + 20";

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        debug: true,
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let parsed_text = parser.parse(&text);

    match parsed_text {
        Ok(tree) => {
            tree.print();
            tree.pretty_print();
            println!("Total: {}", calculate(&tree));
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
