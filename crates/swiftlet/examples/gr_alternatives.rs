use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
    start: hello | namaste
    hello: "Hello"
    namaste: "Namaste"
    "#;

    let texts = ["Hello", "Namaste"];

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf);
    for text in texts {
        match parser.parse(&text) {
            Ok(ast) => {
                ast.print()
                // Output:
                // Tree("start", [Tree("hello", ["Hello"])])
                // Tree("start", [Tree("namaste", ["Namaste"])])
            },
            Err(err) => eprintln!("{}", err),
        }
    }
}
