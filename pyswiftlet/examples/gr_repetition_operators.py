"""Grammar repetition operators example using the Swiftlet Python bindings."""

from swiftlet import Swiftlet


GRAMMAR = r"""
start: number ("+" number)+
number: /\d+/
"""


def main() -> None:
    parser = Swiftlet(GRAMMAR)
    for expr in ("1+2", "1+2+3"):
        tree = parser.parse(expr)
        tree.pretty_print()


if __name__ == "__main__":
    main()
