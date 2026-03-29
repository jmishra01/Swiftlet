use swiftlet::preclude::*;

const GRAMMAR: &str = r#"
    start: column_expr
    column_expr: func
         | column
         | string
         | case
         | literal
         | condition
    func: FUNC_NAME "(" args ")"
    args: column ("," column)*
    case: "case"i when_stmt+ else_stmt? END
    when_stmt: "when"i column_expr "then"i column_expr
    else_stmt: "else"i column_expr
    END: "end"i
    condition: column comparator literal
    comparator: "=" -> eq
        | "!=" -> ne
        | ">=" -> ge
        | "<=" -> le
        | "<" -> gt
        | ">" -> lt
    string: "'" sentence+ "'"
    sentence: NAME | literal
    column: NAME
    literal: INT | DECIMAL
    FUNC_NAME: /[a-zA-Z][a-zA-Z_]+]/
    NAME: /[a-zA-Z][a-zA-Z1-9_]+/
    %import (WS, INT, DECIMAL)
    %ignore WS
    "#;

fn main() {
    let parser_opt = Arc::new(
        ParserOption {
            debug: true,
            algorithm: Algorithm::CLR,
            ..Default::default()
        }
    );
    match Swiftlet::from_string(GRAMMAR, parser_opt) {
        Ok(parser) => {
            let text = "case when Sales > 10 then 'Greater than 10' else 'Less than 10' end";
            match parser.parse(text) {
                Ok(ast) => {
                    ast.pretty_print();
                },
                Err(err) => {
                    println!("{}", err);
                }
            }
        },
        Err(err) => {
            eprintln!("{}", err);
        }
    }


}