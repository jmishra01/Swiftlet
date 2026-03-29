from swiftlet import Swiftlet, Transformer

class Calculate(Transformer):
    def start(self, child):
        return child[0]

    def expr(self, child):
        return child[0]

    def add(self, child):
        return child[0] + child[2]

    def sub(self, child):
        return child[0] - child[2]

    def mul(self, child):
        return child[0] * child[2]

    def div(self, child):
        return child[0] / child[1]

    def terms(self, child):
        return child[0]

def main():
    grammar = """
    start: expr
    expr: expr "+" terms -> add
        | expr "-" terms -> sub
        | terms
    terms: terms "*" INT -> mul
        | terms "/" INT -> div
        | INT
    %import (WS, INT)
    %ignore WS
    """

    text = "12 + 10 - 8 * 2 + 4"
    parser = Swiftlet(grammar)
    parsed_text = parser.parse(text)

    calculate = Calculate()
    result = calculate(parsed_text)

    parsed_text.pretty_print()
    print("\nResult: ", result)


if __name__ == "__main__":
    main()