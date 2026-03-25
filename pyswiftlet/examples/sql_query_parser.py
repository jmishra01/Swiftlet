"""Example SQL query parser built with the Swiftlet Python bindings."""

from __future__ import annotations

from swiftlet import Swiftlet, Transformer


SQL_GRAMMAR = r"""
start: select_stmt
select_stmt: "SELECT"i columns "FROM"i table where_clause?
columns: "*" -> all_columns
    | column ("," column)*
column: NAME
table: NAME

where_clause: "WHERE"i condition
condition: column comparator literal
comparator: "=" -> eq
    | "!=" -> ne
    | ">=" -> ge
    | "<=" -> le
    | ">" -> gt
    | "<" -> lt

literal: string_literal
    | number
string_literal: STRING
number: DECIMAL | INT

NAME: /[a-zA-Z_][a-zA-Z0-9_]*/
%import (DECIMAL, INT, STRING, WS)
%ignore WS
"""


class SqlTransformer(Transformer):
    """Convert the parse tree into a small query description dictionary."""

    @staticmethod
    def start(children):
        return children[0]

    @staticmethod
    def select_stmt(children):
        result = {
            "type": "select",
            "columns": children[1],
            "from": children[3],
        }
        if len(children) == 5:
            result["where"] = children[4]
        return result

    @staticmethod
    def all_columns(_children):
        return ["*"]

    @staticmethod
    def columns(children):
        return children[::2]

    @staticmethod
    def column(children):
        return children[0]

    @staticmethod
    def table(children):
        return children[0]

    @staticmethod
    def where_clause(children):
        return children[1]

    @staticmethod
    def condition(children):
        return {
            "left": children[0],
            "op": children[1],
            "right": children[2],
        }

    @staticmethod
    def eq(_children):
        return "="

    @staticmethod
    def ne(_children):
        return "!="

    @staticmethod
    def ge(_children):
        return ">="

    @staticmethod
    def le(_children):
        return "<="

    @staticmethod
    def gt(_children):
        return ">"

    @staticmethod
    def lt(_children):
        return "<"

    @staticmethod
    def literal(children):
        return children[0]

    @staticmethod
    def string_literal(children):
        return children[0][1:-1]

    @staticmethod
    def number(children):
        return children[0]


def parse_sql(query: str, *, algorithm: str = "clr"):
    """Parse a small subset of SQL SELECT queries."""
    import time
    t1 = time.time()
    parser = Swiftlet(SQL_GRAMMAR, algorithm=algorithm)
    tree = parser.parse(query)
    t2 = time.time()
    print("parsing time: ", t2 - t1)
    return SqlTransformer()(tree)


def main() -> None:
    query = 'SELECT id, name, salary FROM employees WHERE salary >= 5000'
    parsed = parse_sql(query)
    print(parsed)


if __name__ == "__main__":
    main()
