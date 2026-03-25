use std::sync::Arc;
use std::time::Instant;
use fancy_regex::internal::Insn;
use swiftlet::{ParserOption, Swiftlet};
use swiftlet::grammar::Algorithm;


fn main() {
    let grammar = r#"
    start: select_stmt
    select_stmt: "SELECT"i columns "FROM"i table where_clause?
    columns: column ("," column)*
    column: CNAME
    table: CNAME

    where_clause: "where"i condition
    condition: column comparator literal
    comparator: "=" -> eq
        | "!=" -> ne
        | ">=" -> ge
        | "<=" -> le

    literal: INT | DECIMAL

    %import (DECIMAL, INT, WS, CNAME, STRING)
    %ignore WS
    "#;
    let query = r#"SELECT hello from mytable where world >= 120"#;
    let current = Instant::now();
    let conf = Arc::new( ParserOption {algorithm: Algorithm::CLR, debug: true, ..Default::default() });
    let parser = Swiftlet::from_string(grammar, conf);

    let parsed = parser.parse(query);
    println!("Time elapsed: {:?}", current.elapsed());
    match  parsed{
        Ok(ast) => {
            ast.pretty_print();
        }
        Err(err) => eprintln!("Error: {err}"),
    }
}
