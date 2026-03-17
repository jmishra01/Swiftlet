use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
    start: "hello"i
    "#;

    let texts = ["hello", "Hello", "HELLO", "HeLLo"];

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf);
    for text in texts {
        match parser.parse(&text) {
            Ok(ast) => {
                ast.print()
                // Output:
                // Tree("start", ["hello"])
                // Tree("start", ["Hello"])
                // Tree("start", ["HELLO"])
                // Tree("start", ["HeLLo"])

            },
            Err(err) => eprintln!("{}", err),
        }
    }
}
