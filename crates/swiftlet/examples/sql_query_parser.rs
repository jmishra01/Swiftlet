use std::sync::Arc;
use std::time::Instant;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn parse_text(grammar: &str, text: &str) {
    let current = Instant::now();
    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    let parsed = parser.parse(text);
    println!("Time elapsed: {:?}", current.elapsed());
    match parsed {
        Ok(ast) => {
            ast.pretty_print();
        }
        Err(err) => eprintln!("Error: {err}"),
    }
}

fn main() {
    let grammar = r#"
    start: select_stmt
    select_stmt: "SELECT"i columns "FROM"i table where_clause?
    columns: column ("," column)*
    column: NAME
    table: ( NAME | "(" select_stmt ")" )

    where_clause: "where"i condition
    condition: column comparator literal
    comparator: "=" -> eq
        | "!=" -> ne
        | ">=" -> ge
        | "<=" -> le

    literal: INT | DECIMAL

    NAME: /[a-zA-Z_][a-zA-Z0-9_]*/

    %import (DECIMAL, INT, WS, CNAME, STRING)
    %ignore WS
    "#;
    let query = r#"SELECT hello from (select world from mytble) where world >= 120"#;
    parse_text(grammar, query);
}
