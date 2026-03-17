use swiftlet::{Swiftlet, ParserOption};
use std::sync::Arc;

fn main() {
    let grammar = r#"
    start: hello world
    hello: "hello"
    world: "world"
    "#;

    let text = "helloworld";

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf);
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", [Tree("hello", ["hello"]), Tree("world", ["world"])])
        },
        Err(err) => eprintln!("{}", err),
    }

}
