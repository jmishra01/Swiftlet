use std::sync::Arc;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammar = r#"
    start: number ("+" number)+
    number: /\d+/
    "#;

    let math_expr = ["1+2", "1+2+3"];

    let conf = Arc::new(ParserOption::default());
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    for expr in math_expr {
        match parser.parse(&expr) {
            Ok(ast) => {
                ast.print()
                // Output:
                // Tree("start", [Tree("number", ["1"]), "+", Tree("number", ["2"])])
                // Tree("start", [Tree("number", ["1"]), "+", Tree("number", ["2"]), "+", Tree("number", ["3"])])
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
