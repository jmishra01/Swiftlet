import tempfile
import unittest
from pathlib import Path

from swiftlet import Swiftlet, PyAst


GRAMMAR = """
start: expr
expr: INT
%import (WS, INT)
%ignore WS
"""


class SwiftletBindingTests(unittest.TestCase):
    def test_parse_returns_recursive_ast(self) -> None:
        parser = Swiftlet(GRAMMAR)

        ast = parser.parse("42")

        self.assertIsInstance(ast, PyAst)
        self.assertEqual(ast.kind, "tree")
        self.assertEqual(ast.name, "start")
        self.assertEqual(ast.text, 'Tree("start", [Tree("expr", ["42"])])')

        children = ast.children()
        self.assertEqual(len(children), 1)
        self.assertEqual(children[0].kind, "tree")
        self.assertEqual(children[0].name, "expr")

        expr_children = children[0].children()
        self.assertEqual(len(expr_children), 1)
        self.assertTrue(expr_children[0].is_token())
        self.assertEqual(expr_children[0].value, "42")
        self.assertEqual(expr_children[0].terminal, "INT")

    def test_from_file_builds_parser(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            grammar_path = Path(tmp_dir) / "grammar.lark"
            grammar_path.write_text(GRAMMAR, encoding="utf-8")

            parser = Swiftlet.from_file(str(grammar_path))
            ast = parser.parse("7")

        self.assertEqual(ast.kind, "tree")
        self.assertEqual(ast.name, "start")
        self.assertEqual(ast.children()[0].children()[0].value, "7")

    def test_invalid_algorithm_raises_value_error(self) -> None:
        with self.assertRaisesRegex(ValueError, "invalid algorithm"):
            Swiftlet(GRAMMAR, algorithm="lalr")

    def test_parse_failure_raises_runtime_error_for_tokenization_panic(self) -> None:
        parser = Swiftlet(GRAMMAR)

        with self.assertRaisesRegex(RuntimeError, "Failed during tokenization"):
            parser.parse("abc")


if __name__ == "__main__":
    unittest.main()
