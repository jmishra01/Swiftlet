use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammar = r#"
    start: "hello"i
    "#;

    let texts = ["hello", "Hello", "HELLO", "HeLLo"];

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
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
