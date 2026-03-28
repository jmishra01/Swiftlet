use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammar = r#"
    start: hello | namaste
    hello: "Hello"
    namaste: "Namaste"
    "#;

    let texts = ["Hello", "Namaste"];

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    for text in texts {
        match parser.parse(&text) {
            Ok(ast) => {
                ast.print()
                // Output:
                // Tree("start", [Tree("hello", ["Hello"])])
                // Tree("start", [Tree("namaste", ["Namaste"])])
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
