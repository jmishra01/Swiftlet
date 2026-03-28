
use std::sync::Arc;
use swiftlet::preclude::*;


fn main() {
    let grammar = r#"
    start: cname integer text WORD
    cname: CNAME
    integer: INT
    text: TEXT
    TEXT: "text"
    WORD: "word"

    %import (INT, CNAME, WS, DECIMAL)
    %ignore WS
    "#;

    let text = "apple 123 text word";

    let conf = Arc::new(ParserOption {debug: true, ..Default::default() });
    let parser = match Swiftlet::from_string(grammar, conf) {
        Ok(p) => p,
        Err(err) => panic!("{}", err)
    };
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", [Tree("hello", ["hello"]), Tree("world", ["world"])])
        }
        Err(err) => eprintln!("{}", err),
    }
}
