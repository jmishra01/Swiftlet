
from lark import Lark

grammar = """
start: hello WS world
hello: "hello"
world: "word"
%import common.WS
"""

parser = Lark(grammar, parser='earley')

parsed  = parser.parse("helloword")
print(parsed.pretty())