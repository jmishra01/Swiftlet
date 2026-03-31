from swiftlet import Swiftlet

grammar = """
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
"""
if __name__ == "__main__":
    text = "case when Sales > 10 then 'Greater than 10' else 'Less than 10' end"

    parser = Swiftlet(grammar=grammar)
    parsed = parser.parse(text)
    parsed.pretty_print()