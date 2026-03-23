"""Example JSON parser built with the Swiftlet Python bindings."""

from __future__ import annotations


from swiftlet import Swiftlet, Token, Tree, Transformer


JSON_GRAMMAR = r"""
start: value
value: object_rule
    | array
    | string
    | number
    | "true" -> true
    | "false" -> false
    | "null" -> null

object_rule: "{" members? "}"
members: pair ("," pair)*
pair: string ":" value

array: "[" elements? "]"
elements: value ("," value)*

string: STRING
number: "-"? numeric
numeric: DECIMAL | INT

%import (DECIMAL, INT, STRING, WS)
%ignore WS
"""


class JsonTransformer(Transformer):
    """Convert a Swiftlet parse tree into native Python JSON values."""
    @staticmethod
    def string(child):
        ret = child[0][1:-1]
        return ret

    @staticmethod
    def null(_):
        return None

    @staticmethod
    def true(_):
        return True

    @staticmethod
    def false(_):
        return False

    @staticmethod
    def number(child):
        return child[0]

    @staticmethod
    def numeric(child):
        return child[0]

    @staticmethod
    def value(child):
        return child[0]

    @staticmethod
    def pair(child):
        return child[0], child[2]

    @staticmethod
    def elements(child):
        return child[::2]

    @staticmethod
    def array(child):
        return child[1]

    @staticmethod
    def members(child):
        return child[::2]

    @staticmethod
    def object_rule(child):
        return dict(child[1])

    @staticmethod
    def start(child):
        return child[0]



def parse_json(text: str, *, algorithm: str = "earley"):
    """Parse JSON text with Swiftlet and return native Python values."""


def main() -> None:
    sample = """
    {
        "name": "swiftlet",
        "stars": 10,
        "active": true,
        "tags": ["parser", "rust"],
        "meta": {"stable": false},
        "missing": null
    }
    """

    parser = Swiftlet(JSON_GRAMMAR)
    tree = parser.parse(sample)
    parsed = JsonTransformer()(tree)
    print(parsed)


if __name__ == "__main__":
    main()
