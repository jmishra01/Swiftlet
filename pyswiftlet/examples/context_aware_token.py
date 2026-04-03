from swiftlet import Swiftlet

grammar = """
start: hello world
hello: HELLO
world: WORD
HELLO: "hello"i 
WORD: /\w+/
"""


if __name__ == '__main__':
    parser = Swiftlet(grammar, algorithm="earley", debug=True)
    parsed = parser.parse("helloworld")

    print("\n")

    parsed.pretty_print()