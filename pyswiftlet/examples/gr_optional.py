"""Grammar optional example using the Swiftlet Python bindings."""

from swiftlet import Swiftlet


GRAMMAR = r"""
start: hello world
hello: "hello"
world: "world"
name: /\w+/
"""


def main() -> None:
    parser = Swiftlet(GRAMMAR)
    tree = parser.parse("helloworld")
    tree.pretty_print()


if __name__ == "__main__":
    main()
