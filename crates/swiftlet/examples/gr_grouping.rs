use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammar = r#"
    start: (hello | namaster) world
    hello: "Hello"
    namaster: "Namaste"
    world: "World"
    "#;

    let texts = ["HelloWorld", "NamasteWorld"];

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    for text in texts {
        match parser.parse(&text) {
            Ok(ast) => {
                ast.print()
                // Output: Tree("start", [Tree("hello", ["hello"]), Tree("world", ["world"])])
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
