use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
    start: number "+" number
    number: /\d+/
    "#;

    let text = "123+456";

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf);
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", [Tree("number", ["123"]), "+", Tree("number", ["456"])])
        },
        Err(err) => eprintln!("{}", err),
    }

}
