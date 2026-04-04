import tempfile
import unittest
from pathlib import Path

from swiftlet import Swiftlet


class ErrorTests(unittest.TestCase):
    def test_clr_returns_conflict_for_ambiguous_grammar(self) -> None:
        grammar = r"""
        start: expr
        expr: expr "+" expr
            | INT
        %import (WS, INT)
        %ignore WS
        """
        parser = Swiftlet(grammar, algorithm="clr")

        with self.assertRaisesRegex(ValueError, "conflict"):
            parser.parse("1 + 2 + 3")

    def test_clr_returns_rule_not_found_for_unknown_start_token(self) -> None:
        grammar = r"""
        start: expr
        expr: INT
        %import (WS, INT)
        %ignore WS
        """
        parser = Swiftlet(grammar, algorithm="clr")

        with self.assertRaisesRegex(ValueError, "Didn't find any rule for word"):
            parser.parse("abc")

    def test_clr_returns_rule_not_found_for_incomplete_expression(self) -> None:
        grammar = r"""
        start: expr
        expr: INT "+" INT
        %import (WS, INT)
        %ignore WS
        """
        parser = Swiftlet(grammar, algorithm="clr")

        with self.assertRaisesRegex(ValueError, "Didn't find any rule for word"):
            parser.parse("42 +")

    def test_from_file_returns_error_for_missing_grammar_file(self) -> None:
        missing_path = Path(tempfile.gettempdir()) / "swiftlet_missing_grammar_file.lark"

        with self.assertRaisesRegex(ValueError, "Failed to read grammar file"):
            Swiftlet.from_file(str(missing_path), algorithm="clr")
