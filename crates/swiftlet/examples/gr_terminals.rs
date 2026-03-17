use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
    start: HELLO WORLD
    HELLO: "hello"
    WORLD: "world"
    "#;

    let text = "helloworld";

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf);
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", ["hello", "world"])
        },
        Err(err) => eprintln!("{}", err),
    }

}
