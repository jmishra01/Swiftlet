use std::sync::Arc;
use std::time::Instant;
use swiftlet::preclude::*;

const GRAMMAR: &str = r#"
    start: column_expr
    column_expr: func
         | column
         | case
         | literal
         | condition
         | NULL
    func: func_name "(" args ")"
    args: column_expr ("," column_expr)*
    case: "case"i when_stmt+ else_stmt? "end"i
    when_stmt: "when"i column_expr "then"i column_expr
    else_stmt: "else"i column_expr
    condition: column_expr comparator column_expr
    comparator: "=" -> eq
        | "!=" -> ne
        | ">=" -> ge
        | "<=" -> le
        | "<" -> gt
        | ">" -> lt
    column: NAME
    func_name: NAME
    literal: INT | DECIMAL | STRING
    STRING: /'[a-zA-Z0-9_ ]+'/
    NAME: /[a-zA-Z][a-zA-Z1-9_]+/
    %import (WS, INT, DECIMAL)
    %ignore WS
    "#;

const TEXTS: [&str; 10] = [
    "SUM(Sales)",
    "IF_NULL(Sales, 1)",
    "IF_ZERO(Sales, NULL)",
    "SUM(Sales > 5)",
    "SUM(CASE WHEN Sales > 5 THEN 1 ELSE 2 END)",
    "SUM(Sales) > 20",
    "SUM(Cost_Price) > SUM(Selling_Price)",
    "CASE WHEN Sales > 10 THEN 'Greater than 10' ELSE 'Less than 10' END",
    "CASE WHEN Sales > 10 THEN 'Greater than 10' END",
    "CASE WHEN SUM(Sales) > 10 THEN 'Aggregate value is greater than 10' ELSE 'Aggregate value is less than or equals to 10' END",
];


fn details() {
    let parser_opt = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    let grammar_time = Instant::now();
    match Swiftlet::from_string(GRAMMAR, parser_opt) {
        Ok(parser) => {
            println!("Grammar build time: {:?}", grammar_time.elapsed());
            let prefix_text = "Column expr: ";
            TEXTS.into_iter().for_each(|text| {
                println!("{}", "-".repeat(text.len() + prefix_text.len()));
                println!("{}{}", prefix_text, text);
                println!("{}", "-".repeat(text.len() + prefix_text.len()));
                let t1 = Instant::now();
                let parsed = parser.parse(text);
                println!("Parsed time: {:?}", t1.elapsed());
                println!("AST =>");
                parsed.unwrap().pretty_print();
                println!("\n");
            });
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}

fn perf() {
    let calculate_instant = |opt: Arc<ParserOption>| {
        let parser = Swiftlet::from_string(GRAMMAR, opt).unwrap();
        TEXTS
            .iter()
            .map(|text| {
                let start = Instant::now();
                let _ = parser.parse(text).unwrap();
                start.elapsed()
            }).collect::<Vec<_>>()
    };
    // Earley
    let earley_durations = calculate_instant(Arc::new(ParserOption::default()));
    let clr_durations = calculate_instant(Arc::new(ParserOption {algorithm: Algorithm::CLR, ..Default::default()}));
    println!("{}       | {}          | {}", "Earley", "CLR", "Text");
    println!("{}", "-".repeat(100));
    for ((text, earley_time), clr_time) in TEXTS.iter().zip(
        earley_durations.iter()
    ).zip(clr_durations.iter()) {
        println!("{:<12?} | {:<12?} | {}", earley_time, clr_time, text);
    }
}

fn main() {
    details();
    perf();
}
