from lark import Lark
import random
import timeit
from functools import partial

def text_gen():
    operators = ["+", "-", "*", "/"]
    expression = str(random.randint(1, 20))

    for _ in range(100):
        op = random.choice(operators)
        num = random.randint(1, 20)
        expression += f" {op} {num}"

    return expression

grammar = """
start: expr
expr: expr "+" INT -> add
    | expr "-" INT -> sub
    | expr "*" INT -> mul
    | expr "/" INT -> div
    | INT
%import common.INT
%import common.WS
%ignore WS
"""

text_g = text_gen()


def run(text: str):
    lark = Lark(grammar, parser="lalr")
    parsed = lark.parse(text)

call_fn = partial(run, text_g)

print(timeit.timeit(call_fn, number=1))
