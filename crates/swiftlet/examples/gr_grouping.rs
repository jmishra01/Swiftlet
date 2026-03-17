use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
    start: (hello | namaster) world
    hello: "Hello"
    namaster: "Namaste"
    world: "World"
    "#;

    let texts = ["HelloWorld", "NamasteWorld"];

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf);
    for text in texts {
        match parser.parse(&text) {
            Ok(ast) => {
                ast.print()
                // Output: Tree("start", [Tree("hello", ["hello"]), Tree("world", ["world"])])
            },
            Err(err) => eprintln!("{}", err),
        }
    }

}
