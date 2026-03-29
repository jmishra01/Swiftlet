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
    condition: column_expr comparator literal
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

fn main() {
    let parser_opt = Arc::new(
        ParserOption {
            algorithm: Algorithm::CLR,
            ..Default::default()
        }
    );
    match Swiftlet::from_string(GRAMMAR, parser_opt) {
        Ok(parser) => {
            let texts = [
                "sum(Sales)",
                "IF_NULL(Sales, 1)",
                "IF_ZERO(Sales, NULL)",
                "sum(Sales > 5)",
                "sum(case when Sales > 5 then 1 else 2 end)",
                "sum(Sales) > 20",
                "case when Sales > 10 then 'Greater than 10' else 'Less than 10' end",
                "case when Sales > 10 then 'Greater than 10' end",
                "case when sum(Sales) > 10 then 'Aggregate value is greater than 10' else 'Aggregate value is less than or equals to 10' end",
            ];
            for text in texts {
                println!("{}", "-".repeat(text.len() + 6));
                println!("Text: {}", text);
                println!("{}", "-".repeat(text.len() + 6));
                let parsed = parser.parse(text);
                println!("AST =>");
                parsed.unwrap().pretty_print();
                println!("\n");
            }
        },
        Err(err) => {
            eprintln!("{}", err);
        }
    }


}