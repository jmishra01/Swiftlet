use swiftlet::preclude::*;
fn main() {
    let g = r#"
    start: stmt
    stmt: "case"i when_stmt+ else_stmt? "end"i
    when_stmt: "when"i NAME "then"i single_qoute
    else_stmt: "else"i single_qoute
    single_qoute: "'" NAME "'"
    NAME: /[a-zA-Z][a-zA-Z0-9_]*/
    %import WS
    %ignore WS
    "#;
    let text = "case when Sales then 'hello' else 'world' end";

    let parse_opt = Arc::new(ParserOption {
        debug: true,
        ..Default::default()
    });
    let parser = Swiftlet::from_string(g, parse_opt).unwrap();
    match parser.parse(text) {
        Ok(parsed) => {
            println!("{} AST {}", "-".repeat(10), "-".repeat(10));
            parsed.pretty_print();
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
