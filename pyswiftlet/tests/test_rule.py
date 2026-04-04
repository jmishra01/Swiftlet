import unittest

from helpers import make_parse_test


RULE_CASES = {
    "test_rule_text": (
        r"""
        s: "a"
        """,
        "a",
        "s",
    ),
    "test_rule_char_repeat": (
        r"""
        s: a+
        a: "a"
        """,
        "aaa",
        "s",
    ),
    "test_rule_alias": (
        r"""
        s: a+ -> x
        a: "a"
        """,
        "aaa",
        "s",
    ),
    "test_rule_repeat_expr": (
        r"""
        s: e
        e: e "-" N | N
        N: /\d+/
        """,
        "1-23-456-78-9",
        "s",
    ),
    "test_rule_opt": (
        r"""
        s: A b?
        A: "x"
        b: B
        B: "y"
        """,
        "x",
        "s",
    ),
    "test_rule_opt_with_char": (
        r"""
        s: A b?
        A: "x"
        b: B
        B: "y"
        """,
        "xy",
        "s",
    ),
    "test_rule_context_aware": (
        r"""
        s: "A" r
        r: /\w/
        """,
        "AB",
        "s",
    ),
    "test_rule_next_line": (
        r"""
        s: "A" _NL "B"
        _NL: /(\r|\n|\s)*/
        """,
        ("AB", "A\nB", "A\n        B"),
        "s",
    ),
    "test_rule_alternatives": (
        r"""
        s: hello | namaste
        hello: "Hello"
        namaste: "Namaste"
        """,
        ("Hello", "Namaste"),
        "s",
    ),
    "test_rule_grouped_alternatives": (
        r"""
        s: (hello | namaste) world
        hello: "Hello"
        namaste: "Namaste"
        world: "World"
        """,
        ("HelloWorld", "NamasteWorld"),
        "s",
    ),
    "test_rule_group_repeat_plus": (
        r"""
        s: number ("+" number)+
        number: /\d+/
        """,
        "1+2+3",
        "s",
    ),
    "test_rule_group_repeat_star": (
        r"""
        s: number ("+" number)*
        number: /\d+/
        """,
        ("1", "1+2+3"),
        "s",
    ),
    "test_rule_alias_alternatives": (
        r"""
        s: expr
        expr: expr "+" INT -> add
            | expr "-" INT -> sub
            | INT
        %import (WS, INT)
        %ignore WS
        """,
        "3 + 10 - 5 + 20",
        "s",
    ),
    "test_rule_common_numeric_terminals": (
        r"""
        s: integer ":" signed ":" negative ":" decimal
        integer: INT
        signed: SIGNED_INT
        negative: N_INT
        decimal: DECIMAL
        %import (INT, SIGNED_INT, N_INT, DECIMAL)
        """,
        "123:+42:- 9:12.34",
        "s",
    ),
    "test_rule_common_word_and_quote_terminals": (
        r"""
        s: name word lower upper text quoted
        name: CNAME
        word: WORD
        lower: LCASE_LETTER
        upper: UCASE_LETTER
        text: STRING
        quoted: QUOTE
        %import (CNAME, WORD, LCASE_LETTER, UCASE_LETTER, STRING, QUOTE, WS)
        %ignore WS
        """,
        r"""swiftlet1 parser a Z "value" 'x'""",
        "s",
    ),
    "test_rule_common_digit_and_hex_terminals": (
        r"""
        s: DIGIT ":" HEXDIGIT
        %import (DIGIT, HEXDIGIT)
        """,
        "7:BEEF42",
        "s",
    ),
    "test_rule_common_comment_and_newline_terminals": (
        r"""
        s: COMMENT _NL assignment _NL
        assignment: key "=" value
        key: CNAME
        value: STRING
        COMMENT: SH_COMMENT
        _NL: NEWLINE
        %import (CNAME, STRING, SH_COMMENT, NEWLINE, WS_INLINE)
        %ignore WS_INLINE
        """,
        "# service settings\nHOST=\"localhost\"\n",
        "s",
    ),
    "test_rule_common_cr_lf_terminals": (
        r"""
        s: line_end WORD
        line_end: CR LF | LF
        %import (CR, LF, WORD)
        """,
        ("\r\nReady", "\nReady"),
        "s",
    ),
}


class RuleTests(unittest.TestCase):
    pass


for test_name, (grammar, texts, start) in RULE_CASES.items():
    setattr(RuleTests, test_name, make_parse_test(grammar, texts, start))

