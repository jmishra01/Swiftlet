# Swiftlet [![PyPI Downloads](https://static.pepy.tech/personalized-badge/swiftlet?period=total&units=INTERNATIONAL_SYSTEM&left_color=BLACK&right_color=GREEN&left_text=downloads)](https://pepy.tech/projects/swiftlet)


`Swiftlet` provides Python bindings for Swiftlet, a high-performance parsing library built in Rust.
It accepts an EBNF-style grammar and parses input text into a tree of `Tree` and `Token` nodes.

## Features

- Parse context-free grammars from a string or file.
- Choose the parsing algorithm with `earley` or `clr`.
- Control ambiguity handling with `resolve` or `explicit`.
- Inspect the parse result as `Tree` and `Token` objects from Python.
- Context-aware tokenization support, including parser-guided terminal selection for ambiguous token sets.

## Installation

Build and install the package from the `swiftlet` directory:

```bash
pip install swiftlet
```

## Example

```python
from swiftlet import Swiftlet, Transformer

class Calculate(Transformer):
    def start(self, child):
        return child[0]
    
    def expr(self, child):
        return child[0]
    
    def add(self, child):
        return child[0] + child[2]
    
    def sub(self, child):
        return child[0] - child[2]
    
    def mul(self, child):
        return child[0] * child[2]
    
    def div(self, child):
        return child[0] / child[1]
    
    def terms(self, child):
        return child[0]

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

    text = "12 + 10 - 8 * 2 + 4"
    parser = Swiftlet(grammar)
    parsed_text = parser.parse(text)
    
    calculate = Calculate()
    result = calculate(parsed_text)

    parsed_text.pretty_print()
    print("\nResult: ", result)


if __name__ == "__main__":
    main()
```
**Output**
```
start
   add
     sub
       add
         expr
           terms   12
         +
         terms   10
       -
       mul
         terms   8
         *
         2
     +
     terms   4

Result:  10
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
