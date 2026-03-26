"""Grammar terminals example using the Swiftlet Python bindings."""

from swiftlet import Swiftlet


GRAMMAR = r"""
start: HELLO WORLD
HELLO: "hello"
WORLD: "world"
"""


def main() -> None:
    parser = Swiftlet(GRAMMAR)
    tree = parser.parse("helloworld")
    tree.pretty_print()


if __name__ == "__main__":
    main()
