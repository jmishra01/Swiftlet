use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};
use std::sync::Arc;


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

multi_test!(
    rule_clr_sql_column_expr,
    rule_earley_sql_column_expr,
    r#"
    start: column_expr
    column_expr: func
         | column
         | string
         | case
         | literal
         | condition
    func: FUNC_NAME "(" args ")"
    args: column ("," column)*
    case: "case"i ("when"i column_expr "then" column_expr)+ ("else" column_expr)? "end"
    condition: column_expr comparator literal
    comparator: "=" -> eq
        | "!=" -> ne
        | ">=" -> ge
        | "<=" -> le
        | "<" -> gt
        | ">" -> lt
    string: "'" sentence "'"
    sentence: NAME NAME*
    column: NAME
    literal: INT | DECIMAL
    FUNC_NAME: /[a-zA-Z][a-zA-Z_]+]/
    NAME: /[a-zA-Z][a-zA-Z1-9_]+/
    %import (WS, INT, DECIMAL)
    %ignore WS
    "#,
    "case Sales > 10 then 'Greater than 10' else 'Less than 10' end",
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
    Algorithm::Earley);

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
    Algorithm::Earley);

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
    Algorithm::Earley);
