
from lark import Lark

grammar = """
start: expr
expr: select_stmt columns
select_stmt: "SELECT"i
columns: name ("," name)*
name: CNAME

%import common.CNAME
%import common.WS
%ignore WS
"""

parser = Lark(grammar, parser='lalr')

parsed  = parser.parse("SELECT hello, world")
print(parsed.pretty())