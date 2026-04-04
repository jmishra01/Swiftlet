import unittest

from helpers import make_parse_test


SMALL_PROJECT_CASES = {
    "test_simple_query": (
        r"""
        start: select_stmt
        select_stmt: "select"i columns "from"i table_name
        columns: "*" | _columns
        _columns: column ("," column)*
        column: NAME
        table_name: NAME
        NAME: /[a-zA-Z][a-zA-Z1-9_]+/
        %import WS
        %ignore WS
        """,
        "select col1, col2 from stat_table",
        "start",
    ),
    "test_simple_query_with_sub_query": (
        r"""
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
        """,
        "select col1, col2 from (select col3 as col1, col4 as col2 from stat_table) as tb1",
        "start",
    ),
    "test_sql_column_case_expr": (
        r"""
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
        """,
        (
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
        ),
        "start",
    ),
    "test_add_two_numbers": (
        r"""
        start: expr
        expr: (expr "+")? t
        t: INT
        %import (WS, INT)
        %ignore WS
        """,
        "1 + 2",
        "start",
    ),
    "test_add_multi_numbers": (
        r"""
        start: expr
        expr: expr "+" t | t
        t: INT
        %import (WS, INT)
        %ignore WS
        """,
        "1 + 2 + 3 + 4 + 5",
        "start",
    ),
    "test_bodmas": (
        r"""
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
        """,
        "3 * (2 * (3 + 4) - 5)",
        "start",
    ),
}


class SmallProjectTests(unittest.TestCase):
    pass


for test_name, (grammar, texts, start) in SMALL_PROJECT_CASES.items():
    setattr(SmallProjectTests, test_name, make_parse_test(grammar, texts, start))

