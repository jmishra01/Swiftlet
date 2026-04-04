import tempfile
import unittest
from pathlib import Path

from swiftlet import Swiftlet

from helpers import make_parse_test


TERMINAL_CASES = {
    "test_terminal_inline_terminal_concatenate": (
        r"""
        s: A
        A: "a" "b"
        """,
        "ab",
        "s",
    ),
    "test_terminal_flag_i_text_ab": (
        r"""
        s: A
        A: "a" "b"i
        """,
        ("ab", "aB"),
        "s",
    ),
    "test_terminal_reference": (
        r"""
        s: A
        A: "a" B
        B: "b"
        """,
        "ab",
        "s",
    ),
    "test_terminal_multi_reference": (
        r"""
        s: A
        A: X "a" B
        B: "b"
        X: "x"
        """,
        "xab",
        "s",
    ),
    "test_terminal_range": (
        r"""
        s: A
        A: X R+
        X: "x"
        R: "1".."9"
        """,
        "x12345",
        "s",
    ),
    "test_terminal_or_xb_ab": (
        r"""
        s: A
        A: (X | "a") B
        B: "b"
        X: "x"
        """,
        ("xb", "ab"),
        "s",
    ),
    "test_terminal_or_op_xab": (
        r"""
        s: A
        A: (X | "a")+ B
        B: "b"
        X: "x"
        """,
        ("xab", "xaxxab"),
        "s",
    ),
    "test_terminal_regex": (
        r"""
        s: A
        A: "a" B
        B: /b+c*/
        """,
        ("abb", "abbccc"),
        "s",
    ),
    "test_terminal_regex_flags_i": (
        r"""
        s: A
        A: "a" B
        B: /b+c*/i
        """,
        ("abBcCc", "abb", "abbccc"),
        "s",
    ),
    "test_terminal_priority_keyword_over_identifier": (
        r"""
        s: SELECT NAME
        SELECT.10: "select"
        NAME: /[a-z]+/
        %import WS
        %ignore WS
        """,
        "select users",
        "s",
    ),
    "test_terminal_maybe_ab": (
        r"""
        s: A
        A: "a" B
        B: ["b" "c"]
        """,
        "ab",
        "s",
    ),
    "test_terminal_maybe_ac": (
        r"""
        s: A
        A: "a" B
        B: ["b" "c"]
        """,
        "ac",
        "s",
    ),
    "test_terminal_imported_terminal_aliases": (
        r"""
        s: COMMENT _NL
        COMMENT: SH_COMMENT
        _NL: NEWLINE
        %import (SH_COMMENT, NEWLINE)
        """,
        "# service settings\n",
        "s",
    ),
}


class TerminalTests(unittest.TestCase):
    def test_swiftlet_from_file_parses_input(self) -> None:
        grammar = r"""
        start: expr
        expr: INT
        %import (WS, INT)
        %ignore WS
        """
        with tempfile.TemporaryDirectory() as temp_dir:
            grammar_path = Path(temp_dir) / "swiftlet_test_grammar.lark"
            grammar_path.write_text(grammar, encoding="utf-8")

            parser = Swiftlet.from_file(str(grammar_path), algorithm="clr")
            ast = parser.parse("10")

        self.assertEqual(ast.get_name(), "start")


for test_name, (grammar, texts, start) in TERMINAL_CASES.items():
    setattr(TerminalTests, test_name, make_parse_test(grammar, texts, start))

