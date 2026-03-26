"""Grammar grouping example using the Swiftlet Python bindings."""

from swiftlet import Swiftlet


GRAMMAR = r"""
start: (hello | namaster) world
hello: "Hello"
namaster: "Namaste"
world: "World"
"""


def main() -> None:
    parser = Swiftlet(GRAMMAR)
    for text in ("HelloWorld", "NamasteWorld"):
        tree = parser.parse(text)
        tree.pretty_print()


if __name__ == "__main__":
    main()
