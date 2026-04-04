use std::sync::Arc;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let grammar = r#"
    start: hello | namaste
    hello: "Hello"
    namaste: "Namaste"
    "#;

    let texts = ["Hello", "Namaste"];

    let conf = Arc::new(ParserConfig::default());
    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
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
