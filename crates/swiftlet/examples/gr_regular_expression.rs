use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammar = r#"
    start: number "+" number
    number: /\d+/
    "#;

    let text = "123+456";

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", [Tree("number", ["123"]), "+", Tree("number", ["456"])])
        }
        Err(err) => eprintln!("{}", err),
    }
}
