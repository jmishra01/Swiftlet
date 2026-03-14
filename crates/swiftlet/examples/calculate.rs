use swiftlet::{Swiftlet, ParserOption, lexer::AST};
use std::sync::Arc;
use std::time::Instant;
use swiftlet::grammar::Algorithm;


struct Transformer;

impl Transformer {
    fn start(&self, children: &[AST]) -> i64 {
        self.transform(&children[0])
    }

    fn add(&self, children: &[AST]) -> i64 {
        self.transform(&children[0]) + self.transform(&children[2])
    }

    fn sub(&self, children: &[AST]) -> i64 {
        self.transform(&children[0]) - self.transform(&children[2])
    }

    fn mul(&self, children: &[AST]) -> i64 {
        self.transform(&children[0]) * self.transform(&children[2])
    }

    fn div(&self, children: &[AST]) -> i64 {
        self.transform(&children[0]) / self.transform(&children[2])
    }

    fn transform(&self, ast: &AST) -> i64 {
        match ast {
            AST::Token(token) => token.word().parse::<i64>().unwrap(),
            AST::Tree(rule, children) => {
                match rule.as_str() {
                    "start" | "expr" | "term" => self.start(children),
                    "add" => self.add(children),
                    "sub" => self.sub(children),
                    "mul" => self.mul(children),
                    "div" => self.div(children),
                    _ => panic!("{} Rule not found in the AST tree", rule),
                }
            }

        }
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

    let conf = Arc::new(ParserOption {algorithm: Algorithm::CLR, ..Default::default()});
    let parser = Swiftlet::from_string(grammar, conf);
    let parsed_text = parser.parse(&text);

    let transformer = Transformer{};

    match  parsed_text{
        Ok(tree) => {
            println!("AST");
            tree.pretty_print();
            println!("Total: {}", transformer.transform(&tree));
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
