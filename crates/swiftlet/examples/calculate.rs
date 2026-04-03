use std::sync::Arc;
use std::time::Instant;
use swiftlet::preclude::*;

fn calculate(ast: &AST) -> i32 {
    match ast {
        AST::Token(token) => token.word().parse::<i32>().unwrap(),
        AST::Tree(tree, children) => match tree.as_str() {
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

    let t1 = Instant::now();
    let conf = Arc::new(ParserOption {algorithm: Algorithm::Earley, ..Default::default()});
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to get parser");
    let text = "10 - 2 + 5 - 2";

    println!("T1 - {:?}", t1.elapsed());
    let t2 = Instant::now();
    match parser.parse(text) {
        Ok(tree) => {
            println!("T2 - {:?}", t2.elapsed());
            println!("AST: ");
            tree.pretty_print();
            println!("Total: {}", calculate(&tree));
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
