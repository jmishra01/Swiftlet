# Swiftlet - Text parsing library

Swiftlet is a high-performance text-parsing library for Rust, inspired by Python’s [Lark](https://lark-parser.readthedocs.io/en/stable/index.html).

It accepts a context-free grammar (CFG) as input and generates an efficient parser capable 
of analyzing and validating any text that conforms to the specified grammar.
The parser constructs a well-structured Abstract Syntax Tree (AST), enabling further semantic analysis, 
transformation, or code generation.

Designed with Rust’s performance, and safety in mind, Swiftlet aims to provide a robust, extensible, 
and developer-friendly framework for building custom parsers, interpreters, and compilers with minimal boilerplate.

## Features
* Earley parser, capable to parse any context free grammar (CFG).
  * Use cache, to further optimize parsing.
* Canonical LR (CLR) parser is less powerful than an Earley parser, but it is fast and memory-efficient.
* EBNF base grammar
* Builds a parse-tree (AST) based on the grammar.
* Display each step of parser, when debug set true.
* Line and column tracking.

## Example

```rust
use swiftlet::{Swiftlet, ParserOption, lexer::AST};
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
    let mut parser = Swiftlet::from_string(grammar, conf);
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
```

**Output**
```terminaloutput
AST: Tree("start", [Tree("expr", [Tree("sub", [Tree("expr", [Tree("add", [Tree("expr", [Tree("sub", [Tree("expr", ["10"]), "-", "2"])]), "+", "5"])]), "-", "2"])])])
Total: 11
```

**For more examples, please check examples folder.**


## Grammar reference
Click [here](https://github.com/jmishra01/Swiftlet/blob/main/grammar_reference.md)
