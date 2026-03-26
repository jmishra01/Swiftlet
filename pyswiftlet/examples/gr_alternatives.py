"""Grammar alternatives example using the Swiftlet Python bindings."""

from swiftlet import Swiftlet


GRAMMAR = r"""
start: hello | namaste
hello: "Hello"
namaste: "Namaste"
"""


def main() -> None:
    parser = Swiftlet(GRAMMAR)
    for text in ("Hello", "Namaste"):
        tree = parser.parse(text)
        tree.pretty_print()


if __name__ == "__main__":
    main()
