use std::sync::Arc;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let grammar = r#"
    start: "hello"i
    "#;

    let texts = ["hello", "Hello", "HELLO", "HeLLo"];

    let conf = Arc::new(ParserConfig::default());
    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    for text in texts {
        match parser.parse(&text) {
            Ok(ast) => {
                ast.print()
                // Output:
                // Tree("start", ["hello"])
                // Tree("start", ["Hello"])
                // Tree("start", ["HELLO"])
                // Tree("start", ["HeLLo"])
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
