use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

#[macro_use]
mod common;

multi_test!(
    rule_clr_simple_query,
    rule_earley_simple_query,
    r#"
    start: select_stmt
    select_stmt: "select"i columns "from"i table_name
    columns: "*" | _columns
    _columns: column ("," column)*
    column: NAME
    table_name: NAME
    NAME: /[a-zA-Z][a-zA-Z1-9_]+/
    %import WS
    %ignore WS
    "#,
    "select col1, col2 from stat_table",
    "start",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_simple_query_with_sub_query,
    rule_earley_simple_query_with_sub_query,
    r#"
    start: select_stmt
    select_stmt: "select"i columns "from"i table_name
    columns: "*" | _columns
    _columns: column ("," column)*
    column: NAME ("as" alias)?
    table_name: NAME | "(" start ")" "as"i alias
    alias: NAME
    NAME: /[a-zA-Z][a-zA-Z1-9_]+/
    %import WS
    %ignore WS
    "#,
    "select col1, col2 from (select col3 as col1, col4 as col2 from stat_table) as tb1",
    "start",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    rule_clr_sql_column_case_expr,
    rule_earley_sql_column_case_expr,
    r#"
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
    STRING: /'[a-zA-Z0-9%_ ]+'/
    NAME: /[a-zA-Z][a-zA-Z1-9_]+/
    %import (WS, INT, DECIMAL)
    %ignore WS
    "#,
    [
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
    ],
    "start",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_add_two_numbers,
    rule_earley_add_two_numbers,
    r#"
    start: expr
    expr: (expr "+")? t
    t: INT
    %import (WS, INT)
    %ignore WS
    "#,
    "1 + 2",
    "start",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_add_multi_numbers,
    rule_earley_add_multi_numbers,
    r#"
    start: expr
    expr: expr "+" t | t
    t: INT
    %import (WS, INT)
    %ignore WS
    "#,
    "1 + 2 + 3 + 4 + 5",
    "start",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_bodmas,
    rule_earley_bodmas,
    r#"
    start: expr
    expr: expr "+" factor -> add
        | expr "-" factor -> sub
        | factor
    factor: factor "*" term -> multiply
        | factor "/" term -> divide
        | term
    term: "(" expr ")" | INT
    %import (WS, INT)
    %ignore WS
    "#,
    "3 * (2 * (3 + 4) - 5)",
    "start",
    Algorithm::CLR,
    Algorithm::Earley
);
