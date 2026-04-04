use std::sync::Arc;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let grammar = r#"
    start: number "+" number
    number: /\d+/
    "#;

    let text = "123+456";

    let conf = Arc::new(ParserConfig::default());
    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", [Tree("number", ["123"]), "+", Tree("number", ["456"])])
        }
        Err(err) => eprintln!("{}", err),
    }
}
