use std::sync::Arc;
use swiftlet::{ParserConfig, Swiftlet};

fn main() {
    let grammar = r#"
    start: HELLO WORLD
    HELLO: "hello"
    WORLD: "world"
    "#;

    let text = "helloworld";

    let conf = Arc::new(ParserConfig::default());
    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    match parser.parse(&text) {
        Ok(ast) => {
            ast.print()
            // Output: Tree("start", ["hello", "world"])
        }
        Err(err) => eprintln!("{}", err),
    }
}
