use std::sync::Arc;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let grammar = r#"
    start: hello world
    hello: "hello"
    world: "world"
    "#;

    let text = "helloworld";

    let conf = Arc::new(ParserConfig::default());
    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", [Tree("hello", ["hello"]), Tree("world", ["world"])])
        }
        Err(err) => eprintln!("{}", err),
    }
}
