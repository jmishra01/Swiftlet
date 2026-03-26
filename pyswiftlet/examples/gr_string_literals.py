"""Grammar string literals example using the Swiftlet Python bindings."""

from swiftlet import Swiftlet


GRAMMAR = r"""
start: HELLO "(" WORLD ")"
HELLO: "hello"
WORLD: "world"
"""


def main() -> None:
    parser = Swiftlet(GRAMMAR)
    tree = parser.parse("hello(world)")
    tree.pretty_print()


if __name__ == "__main__":
    main()
