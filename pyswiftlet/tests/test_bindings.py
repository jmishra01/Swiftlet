import io
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

from swiftlet import (
    ExceptionTreeType,
    Swiftlet,
    Token,
    Transformer,
    Tree,
    Visitor,
    __version__,
)


SIMPLE_GRAMMAR = """
start: expr
expr: INT
%import (WS, INT)
%ignore WS
"""

ARITHMETIC_GRAMMAR = """
start: expr
expr: expr "+" INT -> add
    | expr "-" INT -> sub
    | INT
%import (WS, INT)
%ignore WS
"""

CONTEXTUAL_GRAMMAR = """
start: "select" NAME
NAME: /[a-z]+/
%import WS
%ignore WS
"""


class EvalTransformer(Transformer):
    def start(self, children):
        return children[0]

    def expr(self, children):
        return children[0]

    def add(self, children):
        return children[0] + children[2]

    def sub(self, children):
        return children[0] - children[2]


class RecordingVisitor(Visitor):
    def __init__(self):
        self.visited = []

    def start(self, tree):
        self.visited.append((tree.get_name(), len(tree.get_children())))

    def add(self, tree):
        self.visited.append((tree.get_name(), len(tree.get_children())))

    def sub(self, tree):
        self.visited.append((tree.get_name(), len(tree.get_children())))

    def expr(self, tree):
        self.visited.append((tree.get_name(), len(tree.get_children())))


class SwiftletBindingTests(unittest.TestCase):
    def test_module_exports_version(self) -> None:
        self.assertTrue(__version__)

    def test_parse_returns_tree_and_token_structure(self) -> None:
        parser = Swiftlet(SIMPLE_GRAMMAR)

        ast = parser.parse("42")

        self.assertIsInstance(ast, Tree)
        self.assertEqual(ast.get_name(), "start")
        self.assertEqual(ast.get_start(), 0)
        self.assertEqual(ast.get_end(), 2)

        children = ast.get_children()
        self.assertEqual(len(children), 1)
        self.assertIsInstance(children[0], Tree)
        self.assertEqual(children[0].get_name(), "expr")

        expr_children = children[0].get_children()
        self.assertEqual(len(expr_children), 1)
        self.assertIsInstance(expr_children[0], Token)
        self.assertEqual(expr_children[0].get_word(), "42")
        self.assertEqual(expr_children[0].get_terminal(), "INT")
        self.assertEqual(expr_children[0].get_start(), 0)
        self.assertEqual(expr_children[0].get_end(), 2)
        self.assertEqual(expr_children[0].get_line(), 0)

    def test_tree_helper_methods_walk_and_filter_nodes(self) -> None:
        parser = Swiftlet(ARITHMETIC_GRAMMAR)
        ast = parser.parse("10 - 2 + 5")

        subtree_names = [node.get_name() for node in ast.iter_subtree()]
        self.assertEqual(subtree_names, ["start", "add", "sub", "expr"])

        add_nodes = list(ast.find_data("add"))
        self.assertEqual(len(add_nodes), 1)
        self.assertEqual(add_nodes[0].get_name(), "add")

        tokens = list(ast.scan_values(lambda token: token.get_terminal() == "INT"))
        self.assertEqual([token.get_word() for token in tokens], ["10", "2", "5"])

        found_tokens = list(ast.find_token("INT"))
        self.assertEqual([token.get_word() for token in found_tokens], ["10", "2", "5"])

    def test_pretty_print_outputs_tree_shape(self) -> None:
        parser = Swiftlet(SIMPLE_GRAMMAR)
        ast = parser.parse("42")

        buffer = io.StringIO()
        with redirect_stdout(buffer):
            ast.pretty_print()

        self.assertEqual(buffer.getvalue(), " start\n   expr   42\n")

    def test_transformer_evaluates_tree(self) -> None:
        parser = Swiftlet(ARITHMETIC_GRAMMAR)
        ast = parser.parse("10 - 2 + 5")

        result = EvalTransformer()(ast)

        self.assertEqual(result, 13)

    def test_transformer_rejects_non_tree_input(self) -> None:
        with self.assertRaises(ExceptionTreeType):
            EvalTransformer()("not-a-tree")

    def test_visitor_walks_tree_top_down(self) -> None:
        parser = Swiftlet(ARITHMETIC_GRAMMAR)
        ast = parser.parse("10 - 2 + 5")

        visitor = RecordingVisitor()
        visitor(ast)

        self.assertEqual(
            visitor.visited,
            [
                ("start", 1),
                ("add", 3),
                ("sub", 3),
                ("expr", 1),
            ],
        )

    def test_visitor_rejects_non_tree_input(self) -> None:
        with self.assertRaises(ExceptionTreeType):
            RecordingVisitor()(123)

    def test_from_file_builds_parser(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            grammar_path = Path(tmp_dir) / "grammar.lark"
            grammar_path.write_text(SIMPLE_GRAMMAR, encoding="utf-8")

            parser = Swiftlet.from_file(str(grammar_path))
            ast = parser.parse("7")

        self.assertEqual(ast.get_name(), "start")
        self.assertEqual(ast.get_children()[0].get_children()[0].get_word(), "7")

    def test_invalid_algorithm_raises_value_error(self) -> None:
        with self.assertRaisesRegex(ValueError, "invalid algorithm"):
            Swiftlet(SIMPLE_GRAMMAR, algorithm="lalr")

    def test_invalid_ambiguity_raises_value_error(self) -> None:
        with self.assertRaisesRegex(ValueError, "invalid ambiguity"):
            Swiftlet(SIMPLE_GRAMMAR, ambiguity="invalid")

    def test_invalid_lexer_mode_raises_value_error(self) -> None:
        with self.assertRaisesRegex(ValueError, "invalid lexer_mode"):
            Swiftlet(SIMPLE_GRAMMAR, lexer_mode="unknown")

    def test_parse_failure_raises_value_error_for_tokenization_error(self) -> None:
        parser = Swiftlet(SIMPLE_GRAMMAR)

        with self.assertRaisesRegex(ValueError, "Tokenization failed"):
            parser.parse("abc")

    def test_dynamic_lexer_mode_handles_contextual_terminals(self) -> None:
        basic = Swiftlet(CONTEXTUAL_GRAMMAR)
        with self.assertRaises(ValueError):
            basic.parse("select users")

        dynamic = Swiftlet(CONTEXTUAL_GRAMMAR, lexer_mode="dynamic")
        ast = dynamic.parse("select users")

        self.assertEqual(ast.get_name(), "start")

    def test_scannerless_lexer_mode_handles_contextual_terminals(self) -> None:
        scannerless = Swiftlet(CONTEXTUAL_GRAMMAR, lexer_mode="scannerless")
        ast = scannerless.parse("select users")

        self.assertEqual(ast.get_name(), "start")

    def test_clr_dynamic_lexer_mode_handles_contextual_terminals(self) -> None:
        parser = Swiftlet(CONTEXTUAL_GRAMMAR, algorithm="clr", lexer_mode="dynamic")
        ast = parser.parse("select users")

        self.assertEqual(ast.get_name(), "start")

    def test_tokens_returns_python_token_wrappers(self) -> None:
        grammar = """
        s: SELECT NAME
        SELECT.10: "select"
        NAME: /[a-z]+/
        %import WS
        %ignore WS
        """
        parser = Swiftlet(grammar)

        tokens = parser.tokens("select users")

        self.assertEqual([token.get_terminal() for token in tokens], ["SELECT", "NAME"])
        self.assertEqual([token.get_word() for token in tokens], ["select", "users"])
        self.assertEqual([token.get_start() for token in tokens], [0, 7])
        self.assertEqual([token.get_end() for token in tokens], [6, 12])

    def test_print_tokens_outputs_debug_lines(self) -> None:
        grammar = """
        s: SELECT NAME
        SELECT.10: "select"
        NAME: /[a-z]+/
        %import WS
        %ignore WS
        """
        parser = Swiftlet(grammar)

        buffer = io.StringIO()
        with redirect_stdout(buffer):
            parser.print_tokens("select users")

        self.assertEqual(
            buffer.getvalue(),
            'SELECT -> "select" @ 0..6\nNAME -> "users" @ 7..12\n',
        )


if __name__ == "__main__":
    unittest.main()
