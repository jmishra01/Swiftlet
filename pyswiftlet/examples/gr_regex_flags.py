"""Grammar regex flags example using the Swiftlet Python bindings."""

from swiftlet import Swiftlet


GRAMMAR = r"""
start: "hello"i
"""


def main() -> None:
    parser = Swiftlet(GRAMMAR)
    for text in ("hello", "Hello", "HELLO", "HeLLo"):
        tree = parser.parse(text)
        tree.pretty_print()


if __name__ == "__main__":
    main()
