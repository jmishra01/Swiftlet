"""Grammar regular expression example using the Swiftlet Python bindings."""

from swiftlet import Swiftlet


GRAMMAR = r"""
start: number "+" number
number: /\d+/
"""


def main() -> None:
    parser = Swiftlet(GRAMMAR)
    tree = parser.parse("123+456")
    tree.pretty_print()


if __name__ == "__main__":
    main()
