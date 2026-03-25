# Swiftlet

`Swiftlet` provides Python bindings for Swiftlet, a high-performance parsing library built in Rust.
It accepts an EBNF-style grammar and parses input text into a tree of `Tree` and `Token` nodes.

## Features

- Parse context-free grammars from a string or file.
- Choose the parsing algorithm with `earley` or `clr`.
- Control ambiguity handling with `resolve` or `explicit`.
- Inspect the parse result as `Tree` and `Token` objects from Python.

## Installation

Build and install the package from the `swiftlet` directory:

```bash
pip install swiftlet
```

## Example

```python
from swiftlet import Swiftlet


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

    text = "12 + 10 - 8 * 20 + 4"
    parser = Swiftlet(grammar)
    tree = parser.parse(text)

    tree.pretty_print()


if __name__ == "__main__":
    main()
```

## Usage

Create a parser from grammar text:

```python
from swiftlet import Swiftlet

parser = Swiftlet(
    grammar,
    start="start",
    algorithm="earley",
    ambiguity="resolve",
    debug=False,
)
```

Create a parser from a grammar file:

```python
from swiftlet import Swiftlet

parser = Swiftlet.from_file("file_name")
```

Parse text and inspect the returned tree:

```python
tree = parser.parse("42")
print(tree.get_name())
print(tree.get_children()[0].get_children()[0].get_word())
```

## Notes

- `algorithm` accepts `earley` or `clr`.
- `ambiguity` accepts `resolve` or `explicit`.
- `parse()` returns a `Tree` on success and raises `ValueError` or `RuntimeError` on failure.

**For more examples, please check notebooks**
